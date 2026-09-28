// extattr_walk: recursive getattrlistbulk walk that optionally requests the CMNEXT
// attributes (recursive gencount, ext flags, private size, clone id/refcnt).
//
// Usage: extattr_walk ROOT [--mode base|ext] [--dump FILE] [--gencount-list FILE]
//                     [--ctime-min EPOCH] [--max-entries N] [--label TEXT]
//        extattr_walk --probe DIR...   (single getattrlist per DIR, prints ext attrs)
//
// base mode requests what fdu requests today (name, devid, objtype, mtime, ctime, flags,
// fileid, dir/file alloc+datalength). ext mode adds forkattr CMNEXT bits under
// FSOPT_ATTR_CMN_EXTENDED (the kernel ORs that option into every bulk call anyway).
// Never use --dump on real trees: it writes names.
#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/attr.h>
#include <sys/resource.h>
#include <sys/stat.h>
#include <sys/time.h>
#include <sys/vnode.h>
#include <time.h>
#include <unistd.h>

#define ATTR_CMN_ERROR_BIT 0x20000000u
#define BUFSZ (256 * 1024)

static const attrgroup_t COMMON = ATTR_CMN_RETURNED_ATTRS | ATTR_CMN_NAME | ATTR_CMN_DEVID | ATTR_CMN_OBJTYPE |
                                  ATTR_CMN_MODTIME | ATTR_CMN_CHGTIME | ATTR_CMN_FLAGS | ATTR_CMN_FILEID |
                                  ATTR_CMN_ERROR_BIT;
static const attrgroup_t DIRA = ATTR_DIR_MOUNTSTATUS | ATTR_DIR_ALLOCSIZE | ATTR_DIR_DATALENGTH;
static const attrgroup_t FILEA = ATTR_FILE_ALLOCSIZE | ATTR_FILE_DATALENGTH;
static const attrgroup_t EXTA = ATTR_CMNEXT_PRIVATESIZE | ATTR_CMNEXT_CLONEID | ATTR_CMNEXT_EXT_FLAGS |
                                ATTR_CMNEXT_RECURSIVE_GENCOUNT | ATTR_CMNEXT_CLONE_REFCNT;

struct stats {
    unsigned long dirs, files, links, others, errors, declined;
    unsigned long long sum_datalen, sum_alloc, sum_private;
    unsigned long private_returned, ext_flags_returned, cloneid_returned, refcnt_returned, gencount_returned;
    unsigned long gencount_nonzero;
    unsigned long may_share, shares_all, sparse, purgeable, no_xattrs, sync_root;
    unsigned long refcnt_gt1;
    unsigned long long alloc_of_may_share, private_of_may_share;
    unsigned long bulk_calls;
    unsigned long ctime_hits;
};

static struct stats st;
static int mode_ext = 0;
static FILE *dump, *genlist;
static long long ctime_min = -1;
static unsigned long max_entries = 0;
static unsigned long total_entries = 0;
static char *bufp;
static dev_t root_dev;

static double now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1000.0 + ts.tv_nsec / 1e6;
}
static double tv_ms(struct timeval tv) { return tv.tv_sec * 1000.0 + tv.tv_usec / 1000.0; }

struct rd { const char *p; size_t n, pos; int bad; };
static uint32_t rd_u32(struct rd *r) { if (r->pos + 4 > r->n) { r->bad = 1; return 0; } uint32_t v; memcpy(&v, r->p + r->pos, 4); r->pos += 4; return v; }
static int32_t rd_i32(struct rd *r) { return (int32_t)rd_u32(r); }
static uint64_t rd_u64(struct rd *r) { if (r->pos + 8 > r->n) { r->bad = 1; return 0; } uint64_t v; memcpy(&v, r->p + r->pos, 8); r->pos += 8; return v; }
static int64_t rd_i64(struct rd *r) { return (int64_t)rd_u64(r); }

static void walk(int dirfd, const char *relpath, int depth);

static void handle_entry(int dirfd, const char *relpath, int depth, const char *rec, size_t len) {
    struct rd r = { rec, len, 0, 0 };
    uint32_t declared = rd_u32(&r);
    if (declared != len) { st.errors++; return; }
    attribute_set_t ret;
    ret.commonattr = rd_u32(&r); ret.volattr = rd_u32(&r); ret.dirattr = rd_u32(&r);
    ret.fileattr = rd_u32(&r); ret.forkattr = rd_u32(&r);
    if (ret.commonattr & ATTR_CMN_ERROR_BIT) { uint32_t e = rd_u32(&r); if (e) { st.errors++; return; } }
    size_t refpos = r.pos;
    int32_t name_off = rd_i32(&r); uint32_t name_len = rd_u32(&r);
    int32_t dev = (ret.commonattr & ATTR_CMN_DEVID) ? rd_i32(&r) : 0;
    uint32_t objtype = rd_u32(&r);
    int64_t mt_s = rd_i64(&r), mt_ns = rd_i64(&r); (void)mt_ns;
    int64_t ct_s = rd_i64(&r), ct_ns = rd_i64(&r); (void)ct_ns;
    uint32_t flags = rd_u32(&r); (void)flags;
    uint64_t fileid = rd_u64(&r);
    uint32_t mountstatus = 0; int64_t dalloc = 0, dlen = 0, falloc = 0, flen = 0;
    if (ret.dirattr & ATTR_DIR_MOUNTSTATUS) mountstatus = rd_u32(&r);
    if (ret.dirattr & ATTR_DIR_ALLOCSIZE) dalloc = rd_i64(&r);
    if (ret.dirattr & ATTR_DIR_DATALENGTH) dlen = rd_i64(&r);
    if (ret.fileattr & ATTR_FILE_ALLOCSIZE) falloc = rd_i64(&r);
    if (ret.fileattr & ATTR_FILE_DATALENGTH) flen = rd_i64(&r);
    int64_t privsize = -1; uint64_t cloneid = 0, extflags = 0, gencount = 0; uint32_t refcnt = 0;
    int have_gc = 0, have_ef = 0;
    if (ret.forkattr & ATTR_CMNEXT_PRIVATESIZE) { privsize = rd_i64(&r); st.private_returned++; }
    if (ret.forkattr & ATTR_CMNEXT_CLONEID) { cloneid = rd_u64(&r); st.cloneid_returned++; }
    if (ret.forkattr & ATTR_CMNEXT_EXT_FLAGS) { extflags = rd_u64(&r); st.ext_flags_returned++; have_ef = 1; }
    if (ret.forkattr & ATTR_CMNEXT_RECURSIVE_GENCOUNT) { gencount = rd_u64(&r); st.gencount_returned++; have_gc = 1; }
    if (ret.forkattr & ATTR_CMNEXT_CLONE_REFCNT) { refcnt = rd_u32(&r); st.refcnt_returned++; }
    if (r.bad) { st.errors++; return; }
    size_t name_start = refpos + (size_t)name_off;
    if (name_start + name_len > len || name_len == 0) { st.errors++; return; }
    const char *name = rec + name_start;
    if (!strcmp(name, ".") || !strcmp(name, "..")) return;
    total_entries++;

    int is_dir = objtype == VDIR;
    int64_t alloc = is_dir ? dalloc : falloc, dlength = is_dir ? dlen : flen;
    st.sum_datalen += (unsigned long long)dlength;
    st.sum_alloc += (unsigned long long)alloc;
    if (privsize >= 0) st.sum_private += (unsigned long long)privsize;
    if (ctime_min >= 0 && ct_s >= ctime_min) st.ctime_hits++;
    if (have_gc && gencount) {
        st.gencount_nonzero++;
        if (genlist) fprintf(genlist, "%" PRIu64 "\t%d\t%s/%s\n", gencount, depth, relpath, name);
    }
    if (have_ef) {
        if (extflags & EF_MAY_SHARE_BLOCKS) { st.may_share++; st.alloc_of_may_share += (unsigned long long)alloc; if (privsize >= 0) st.private_of_may_share += (unsigned long long)privsize; }
        if (extflags & EF_SHARES_ALL_BLOCKS) st.shares_all++;
        if (extflags & EF_IS_SPARSE) st.sparse++;
        if (extflags & EF_IS_PURGEABLE) st.purgeable++;
        if (extflags & EF_NO_XATTRS) st.no_xattrs++;
        if (extflags & EF_IS_SYNC_ROOT) st.sync_root++;
    }
    if (refcnt > 1) st.refcnt_gt1++;
    if (dump) {
        fprintf(dump, "%" PRIu64 "\t%u\t%" PRId64 "\t%" PRId64 "\t%" PRId64 "\t%" PRId64 "\t%" PRId64 "\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%u\t%s/%s\n",
                fileid, objtype, dlength, alloc, ct_s, mt_s, privsize, cloneid, extflags, gencount, refcnt, relpath, name);
    }
    if (is_dir) {
        st.dirs++;
        if (mountstatus & DIR_MNTSTATUS_MNTPOINT) return;
        if ((ret.commonattr & ATTR_CMN_DEVID) && dev != (int32_t)root_dev) return;
        if (max_entries && total_entries >= max_entries) return;
        char child[4096];
        snprintf(child, sizeof child, "%s/%s", relpath, name);
        int fd = openat(dirfd, name, O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC);
        if (fd < 0) { st.errors++; return; }
        walk(fd, child, depth + 1);
        close(fd);
    } else if (objtype == VREG) st.files++;
    else if (objtype == VLNK) st.links++;
    else st.others++;
}

static void walk(int dirfd, const char *relpath, int depth) {
    struct attrlist al;
    memset(&al, 0, sizeof al);
    al.bitmapcount = ATTR_BIT_MAP_COUNT;
    al.commonattr = COMMON;
    al.dirattr = DIRA;
    al.fileattr = FILEA;
    al.forkattr = mode_ext ? EXTA : 0;
    // Entries are collected first so the buffer is not reused during recursion.
    char *dirbuf = NULL; size_t dirbuf_n = 0, dirbuf_cap = 0;
    for (;;) {
        int n = getattrlistbulk(dirfd, &al, bufp, BUFSZ, FSOPT_ATTR_CMN_EXTENDED);
        if (n < 0) { st.declined++; free(dirbuf); return; }
        st.bulk_calls++;
        if (n == 0) break;
        size_t off = 0;
        for (int i = 0; i < n; i++) {
            uint32_t len; memcpy(&len, bufp + off, 4);
            if (len < 4 || off + len > BUFSZ) { st.errors++; break; }
            if (dirbuf_n + len > dirbuf_cap) { dirbuf_cap = dirbuf_cap ? dirbuf_cap * 2 : 65536; while (dirbuf_n + len > dirbuf_cap) dirbuf_cap *= 2; dirbuf = realloc(dirbuf, dirbuf_cap); }
            memcpy(dirbuf + dirbuf_n, bufp + off, len);
            dirbuf_n += len;
            off += len;
        }
    }
    size_t off = 0;
    while (off < dirbuf_n) {
        uint32_t len; memcpy(&len, dirbuf + off, 4);
        handle_entry(dirfd, relpath, depth, dirbuf + off, len);
        off += len;
    }
    free(dirbuf);
}

static int probe(const char *path) {
    struct attrlist al;
    memset(&al, 0, sizeof al);
    al.bitmapcount = ATTR_BIT_MAP_COUNT;
    al.commonattr = ATTR_CMN_RETURNED_ATTRS | ATTR_CMN_OBJTYPE | ATTR_CMN_FILEID | ATTR_CMN_GEN_COUNT;
    al.forkattr = EXTA;
    char buf[512];
    if (getattrlist(path, &al, buf, sizeof buf, FSOPT_ATTR_CMN_EXTENDED | FSOPT_NOFOLLOW) != 0) {
        printf("%s: getattrlist failed: %s\n", path, strerror(errno));
        return 1;
    }
    struct rd r = { buf, sizeof buf, 0, 0 };
    uint32_t len = rd_u32(&r); (void)len;
    attribute_set_t ret; ret.commonattr = rd_u32(&r); ret.volattr = rd_u32(&r); ret.dirattr = rd_u32(&r); ret.fileattr = rd_u32(&r); ret.forkattr = rd_u32(&r);
    uint32_t objtype = (ret.commonattr & ATTR_CMN_OBJTYPE) ? rd_u32(&r) : 0;
    uint32_t gen = (ret.commonattr & ATTR_CMN_GEN_COUNT) ? rd_u32(&r) : 0;
    uint64_t fileid = (ret.commonattr & ATTR_CMN_FILEID) ? rd_u64(&r) : 0;
    printf("%s: objtype=%u fileid=%" PRIu64 " gen_count=%u returned_fork=0x%x", path, objtype, fileid, gen, ret.forkattr);
    if (ret.forkattr & ATTR_CMNEXT_PRIVATESIZE) printf(" privatesize=%" PRId64, rd_i64(&r));
    if (ret.forkattr & ATTR_CMNEXT_CLONEID) printf(" cloneid=%" PRIu64, rd_u64(&r));
    if (ret.forkattr & ATTR_CMNEXT_EXT_FLAGS) printf(" ext_flags=0x%" PRIx64, rd_u64(&r));
    if (ret.forkattr & ATTR_CMNEXT_RECURSIVE_GENCOUNT) printf(" recursive_gencount=%" PRIu64, rd_u64(&r));
    if (ret.forkattr & ATTR_CMNEXT_CLONE_REFCNT) printf(" clone_refcnt=%u", rd_u32(&r));
    printf("\n");
    return 0;
}

int main(int argc, char **argv) {
    if (argc < 2) { fprintf(stderr, "usage: %s ROOT [--mode base|ext] [--dump FILE] [--gencount-list FILE] [--ctime-min EPOCH] [--max-entries N] [--label TEXT] | --probe DIR...\n", argv[0]); return 2; }
    if (!strcmp(argv[1], "--probe")) { int rc = 0; for (int i = 2; i < argc; i++) rc |= probe(argv[i]); return rc; }
    const char *root = argv[1], *label = "";
    for (int i = 2; i < argc; i++) {
        if (!strcmp(argv[i], "--mode") && i + 1 < argc) mode_ext = !strcmp(argv[++i], "ext");
        else if (!strcmp(argv[i], "--dump") && i + 1 < argc) dump = fopen(argv[++i], "w");
        else if (!strcmp(argv[i], "--gencount-list") && i + 1 < argc) genlist = fopen(argv[++i], "w");
        else if (!strcmp(argv[i], "--ctime-min") && i + 1 < argc) ctime_min = atoll(argv[++i]);
        else if (!strcmp(argv[i], "--max-entries") && i + 1 < argc) max_entries = strtoul(argv[++i], NULL, 10);
        else if (!strcmp(argv[i], "--label") && i + 1 < argc) label = argv[++i];
        else { fprintf(stderr, "unknown option %s\n", argv[i]); return 2; }
    }
    if (dump) fprintf(dump, "fileid\tobjtype\tdatalen\talloc\tctime\tmtime\tprivatesize\tcloneid\textflags\tgencount\trefcnt\tpath\n");
    bufp = malloc(BUFSZ);
    struct stat sb;
    if (lstat(root, &sb) != 0) { perror("lstat root"); return 1; }
    root_dev = sb.st_dev;
    int fd = open(root, O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC);
    if (fd < 0) { perror("open root"); return 1; }
    struct rusage ru0, ru1; getrusage(RUSAGE_SELF, &ru0);
    double t0 = now_ms();
    walk(fd, "", 0);
    double t1 = now_ms();
    getrusage(RUSAGE_SELF, &ru1);
    close(fd);
    if (dump) fclose(dump);
    if (genlist) fclose(genlist);
    unsigned long entries = st.dirs + st.files + st.links + st.others;
    printf("label=%s mode=%s entries=%lu dirs=%lu files=%lu links=%lu others=%lu errors=%lu declined=%lu bulk_calls=%lu "
           "sum_datalen=%llu sum_alloc=%llu sum_private=%llu private_returned=%lu ext_flags_returned=%lu cloneid_returned=%lu "
           "refcnt_returned=%lu gencount_returned=%lu gencount_nonzero=%lu may_share=%lu shares_all=%lu sparse=%lu purgeable=%lu "
           "no_xattrs=%lu sync_root=%lu refcnt_gt1=%lu alloc_of_may_share=%llu private_of_may_share=%llu ctime_hits=%lu "
           "wall_ms=%.1f user_ms=%.1f sys_ms=%.1f us_per_entry=%.3f\n",
           label, mode_ext ? "ext" : "base", entries, st.dirs, st.files, st.links, st.others, st.errors, st.declined, st.bulk_calls,
           st.sum_datalen, st.sum_alloc, st.sum_private, st.private_returned, st.ext_flags_returned, st.cloneid_returned,
           st.refcnt_returned, st.gencount_returned, st.gencount_nonzero, st.may_share, st.shares_all, st.sparse, st.purgeable,
           st.no_xattrs, st.sync_root, st.refcnt_gt1, st.alloc_of_may_share, st.private_of_may_share, st.ctime_hits,
           t1 - t0, tv_ms(ru1.ru_utime) - tv_ms(ru0.ru_utime), tv_ms(ru1.ru_stime) - tv_ms(ru0.ru_stime),
           entries ? (t1 - t0) * 1000.0 / entries : 0.0);
    return 0;
}
