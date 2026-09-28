// ds: independent probe for APFS maintain-dir-stats (verification tool, not for the repo).
//
//   ds gen PATH...              getattrlist: recursive gencount, ext_flags, privatesize, alloc/data sizes
//   ds get PATH [REPEAT]        fsctl 0xC1104A71 with zeroed struct (GET); dump nonzero words; time it
//   ds mark PATH [W1 [W0]]      fsctl 0xC1104A71 with u32@+4=W1 (default 1) and u32@+0=W0 (default 0).
//                               W1=1 is what apfs.util -M does. Other values are experiments: use only on
//                               disposable fixtures.
//   ds walk ROOT [-o OUT] [-p CKPT] [-q]
//                               getattrlistbulk walk. Emits one TSV line per entry to OUT (or nothing with -q):
//                               kind path fileid linkcount datalength dataalloc allocsize rsrcalloc privatesize gencount extflags cloneid clonerefcnt mtime_ns
//                               With -p CKPT (lines "gencount<TAB>relpath"), a directory whose gencount is
//                               nonzero and equals the checkpoint is NOT descended (pruned walk). Prints a
//                               summary line: entries, dirs, files, pruned, wall_ms, bytes.
#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/attr.h>
#include <sys/mount.h>
#include <sys/stat.h>
#include <sys/vnode.h>
#include <time.h>
#include <unistd.h>

#define DS_FSCTL 0xC1104A71u
#define DS_STRUCT 272

static double now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1000.0 + ts.tv_nsec / 1e6;
}

// ---------------------------------------------------------------- getattrlist single path
struct genrec {
    uint64_t gencount, extflags, cloneid;
    int64_t privatesize;
    uint32_t clonerefcnt, objtype;
    uint64_t datalength, dataalloc, allocsize, rsrcalloc;
    uint32_t returned_fork, returned_file;
};

static int read_gen(const char *path, struct genrec *g) {
    struct attrlist al;
    memset(&al, 0, sizeof al);
    al.bitmapcount = ATTR_BIT_MAP_COUNT;
    al.commonattr = ATTR_CMN_RETURNED_ATTRS | ATTR_CMN_OBJTYPE;
    al.fileattr = ATTR_FILE_DATALENGTH | ATTR_FILE_DATAALLOCSIZE | ATTR_FILE_ALLOCSIZE | ATTR_FILE_RSRCALLOCSIZE;
    al.forkattr = ATTR_CMNEXT_PRIVATESIZE | ATTR_CMNEXT_CLONEID | ATTR_CMNEXT_EXT_FLAGS | ATTR_CMNEXT_RECURSIVE_GENCOUNT | ATTR_CMNEXT_CLONE_REFCNT;
    unsigned char buf[512];
    if (getattrlist(path, &al, buf, sizeof buf, FSOPT_ATTR_CMN_EXTENDED | FSOPT_NOFOLLOW) != 0) return -1;
    memset(g, 0, sizeof *g);
    unsigned char *p = buf + 4;
    attribute_set_t ret; memcpy(&ret, p, sizeof ret); p += sizeof ret;
    if (ret.commonattr & ATTR_CMN_OBJTYPE) { memcpy(&g->objtype, p, 4); p += 4; }
    // file attrs (bit order): ALLOCSIZE(0x4), DATALENGTH(0x200), DATAALLOCSIZE(0x400), RSRCALLOCSIZE(0x2000)
    g->returned_file = ret.fileattr;
    if (ret.fileattr & ATTR_FILE_ALLOCSIZE) { memcpy(&g->allocsize, p, 8); p += 8; }
    if (ret.fileattr & ATTR_FILE_DATALENGTH) { memcpy(&g->datalength, p, 8); p += 8; }
    if (ret.fileattr & ATTR_FILE_DATAALLOCSIZE) { memcpy(&g->dataalloc, p, 8); p += 8; }
    if (ret.fileattr & ATTR_FILE_RSRCALLOCSIZE) { memcpy(&g->rsrcalloc, p, 8); p += 8; }
    g->returned_fork = ret.forkattr;
    if (ret.forkattr & ATTR_CMNEXT_PRIVATESIZE) { memcpy(&g->privatesize, p, 8); p += 8; }
    if (ret.forkattr & ATTR_CMNEXT_CLONEID) { memcpy(&g->cloneid, p, 8); p += 8; }
    if (ret.forkattr & ATTR_CMNEXT_EXT_FLAGS) { memcpy(&g->extflags, p, 8); p += 8; }
    if (ret.forkattr & ATTR_CMNEXT_RECURSIVE_GENCOUNT) { memcpy(&g->gencount, p, 8); p += 8; }
    if (ret.forkattr & ATTR_CMNEXT_CLONE_REFCNT) { memcpy(&g->clonerefcnt, p, 4); p += 4; }
    return 0;
}

static int cmd_gen(int argc, char **argv) {
    for (int i = 0; i < argc; i++) {
        struct genrec g;
        if (read_gen(argv[i], &g) != 0) { printf("%s\tERR\t%s\n", argv[i], strerror(errno)); continue; }
        printf("%s\tgencount=%" PRIu64 "\textflags=0x%" PRIx64 "\tprivatesize=%" PRId64 "\tcloneid=%" PRIu64 "\trefcnt=%u\tdatalen=%" PRIu64 "\tdataalloc=%" PRIu64 "\talloc=%" PRIu64 "\trsrcalloc=%" PRIu64 "\tret_fork=0x%x\n",
               argv[i], g.gencount, g.extflags, g.privatesize, g.cloneid, g.clonerefcnt, g.datalength, g.dataalloc, g.allocsize, g.rsrcalloc, g.returned_fork);
    }
    return 0;
}

// ---------------------------------------------------------------- fsctl GET / MARK
static int cmd_get(int argc, char **argv) {
    if (argc < 1) return 2;
    int repeat = argc > 1 ? atoi(argv[1]) : 0;
    unsigned char buf[DS_STRUCT];
    memset(buf, 0, sizeof buf);
    double t0 = now_ms();
    int rc = fsctl(argv[0], DS_FSCTL, buf, 0);
    double t1 = now_ms();
    if (rc != 0) { printf("%s\tfsctl_get_failed\terrno=%d\t%s\n", argv[0], errno, strerror(errno)); return 1; }
    uint64_t gen, desc, phys;
    memcpy(&gen, buf + 0x30, 8); memcpy(&desc, buf + 0x38, 8); memcpy(&phys, buf + 0x40, 8);
    printf("%s\tgen=%" PRIu64 "\tdesc=%" PRIu64 "\tphys=%" PRIu64 "\tfirst_ms=%.4f", argv[0], gen, desc, phys, t1 - t0);
    for (int i = 0; i < DS_STRUCT; i += 8) {
        uint64_t v; memcpy(&v, buf + i, 8);
        if (v && i != 0x30 && i != 0x38 && i != 0x40) printf("\tword+0x%02x=0x%" PRIx64, i, v);
    }
    if (repeat > 0) {
        double best = 1e9, sum = 0;
        for (int i = 0; i < repeat; i++) {
            memset(buf, 0, sizeof buf);
            double a = now_ms();
            if (fsctl(argv[0], DS_FSCTL, buf, 0) != 0) { printf("\tfsctl_repeat_failed errno=%d", errno); break; }
            double d = now_ms() - a;
            sum += d; if (d < best) best = d;
        }
        printf("\trepeat=%d\tmean_ms=%.4f\tmin_ms=%.4f", repeat, sum / repeat, best);
    }
    printf("\n");
    return 0;
}

static int cmd_mark(int argc, char **argv) {
    if (argc < 1) return 2;
    uint32_t w1 = argc > 1 ? (uint32_t)strtoul(argv[1], NULL, 0) : 1;
    uint32_t w0 = argc > 2 ? (uint32_t)strtoul(argv[2], NULL, 0) : 0;
    unsigned char buf[DS_STRUCT];
    memset(buf, 0, sizeof buf);
    memcpy(buf + 4, &w1, 4);
    memcpy(buf + 0, &w0, 4);
    double t0 = now_ms();
    int rc = fsctl(argv[0], DS_FSCTL, buf, 0);
    double t1 = now_ms();
    if (rc != 0) { printf("%s\tmark_failed\tw1=0x%x\tw0=0x%x\terrno=%d\t%s\tms=%.3f\n", argv[0], w1, w0, errno, strerror(errno), t1 - t0); return 1; }
    uint64_t gen, desc, phys;
    memcpy(&gen, buf + 0x30, 8); memcpy(&desc, buf + 0x38, 8); memcpy(&phys, buf + 0x40, 8);
    printf("%s\tmark_ok\tw1=0x%x\tw0=0x%x\tms=%.3f\tgen=%" PRIu64 "\tdesc=%" PRIu64 "\tphys=%" PRIu64, argv[0], w1, w0, t1 - t0, gen, desc, phys);
    for (int i = 0; i < DS_STRUCT; i += 8) {
        uint64_t v; memcpy(&v, buf + i, 8);
        if (v && i != 0x30 && i != 0x38 && i != 0x40) printf("\tword+0x%02x=0x%" PRIx64, i, v);
    }
    printf("\n");
    return 0;
}

// ---------------------------------------------------------------- bulk walk
struct ckpt { char *rel; uint64_t gen; };
static struct ckpt *ckpts; static size_t nck;

static int ck_cmp(const void *a, const void *b) { return strcmp(((const struct ckpt *)a)->rel, ((const struct ckpt *)b)->rel); }

static int load_ckpt(const char *file) {
    FILE *f = fopen(file, "r");
    if (!f) return -1;
    char line[8192]; size_t cap = 1024; ckpts = malloc(cap * sizeof *ckpts);
    while (fgets(line, sizeof line, f)) {
        char *tab = strchr(line, '\t'); if (!tab) continue;
        *tab = 0; char *rel = tab + 1; size_t L = strlen(rel); if (L && rel[L - 1] == '\n') rel[L - 1] = 0;
        if (nck == cap) { cap *= 2; ckpts = realloc(ckpts, cap * sizeof *ckpts); }
        ckpts[nck].gen = strtoull(line, NULL, 10); ckpts[nck].rel = strdup(rel); nck++;
    }
    fclose(f);
    qsort(ckpts, nck, sizeof *ckpts, ck_cmp);
    return 0;
}

static int ck_lookup(const char *rel, uint64_t *gen) {
    size_t lo = 0, hi = nck;
    while (lo < hi) {
        size_t mid = (lo + hi) / 2; int c = strcmp(ckpts[mid].rel, rel);
        if (c == 0) { *gen = ckpts[mid].gen; return 1; }
        if (c < 0) lo = mid + 1; else hi = mid;
    }
    return 0;
}

static FILE *out; static int quiet; static int base_only; static int gen_only;
static uint64_t n_entries, n_dirs, n_files, n_pruned, n_other, tot_alloc, tot_data;
static size_t rootlen;

static void walk(int dfd, char *path, size_t plen) {
    struct attrlist al;
    memset(&al, 0, sizeof al);
    al.bitmapcount = ATTR_BIT_MAP_COUNT;
    al.commonattr = ATTR_CMN_RETURNED_ATTRS | ATTR_CMN_NAME | ATTR_CMN_OBJTYPE | ATTR_CMN_MODTIME | ATTR_CMN_FILEID | ATTR_CMN_ERROR;
    al.fileattr = ATTR_FILE_LINKCOUNT | ATTR_FILE_ALLOCSIZE | ATTR_FILE_DATALENGTH | ATTR_FILE_DATAALLOCSIZE | ATTR_FILE_RSRCALLOCSIZE;
    al.forkattr = base_only ? 0 : gen_only ? (ATTR_CMNEXT_RECURSIVE_GENCOUNT) : (ATTR_CMNEXT_PRIVATESIZE | ATTR_CMNEXT_CLONEID | ATTR_CMNEXT_EXT_FLAGS | ATTR_CMNEXT_RECURSIVE_GENCOUNT | ATTR_CMNEXT_CLONE_REFCNT);
    static __thread unsigned char *buf; static __thread size_t bufsz;
    if (!buf) { bufsz = 256 * 1024; buf = malloc(bufsz); }
    // Collect subdirectory names first, then recurse (keeps one fd per level).
    size_t subcap = 64, nsub = 0; char **subs = malloc(subcap * sizeof *subs);
    for (;;) {
        int n = getattrlistbulk(dfd, &al, buf, bufsz, 0);
        if (n < 0) { fprintf(stderr, "getattrlistbulk %s: %s\n", path, strerror(errno)); break; }
        if (n == 0) break;
        unsigned char *p = buf;
        for (int i = 0; i < n; i++) {
            uint32_t len; memcpy(&len, p, 4);
            unsigned char *q = p + 4;
            attribute_set_t ret; memcpy(&ret, q, sizeof ret); q += sizeof ret;
            uint32_t err = 0; if (ret.commonattr & ATTR_CMN_ERROR) { memcpy(&err, q, 4); q += 4; }
            const char *name = ""; if (ret.commonattr & ATTR_CMN_NAME) { attrreference_t r; memcpy(&r, q, sizeof r); name = (const char *)q + r.attr_dataoffset; q += sizeof r; }
            uint32_t objtype = 0; if (ret.commonattr & ATTR_CMN_OBJTYPE) { memcpy(&objtype, q, 4); q += 4; }
            struct timespec mt = {0, 0}; if (ret.commonattr & ATTR_CMN_MODTIME) { memcpy(&mt, q, sizeof mt); q += sizeof mt; }
            uint64_t fileid = 0; if (ret.commonattr & ATTR_CMN_FILEID) { memcpy(&fileid, q, 8); q += 8; }
            uint32_t linkcount = 0; uint64_t alloc = 0, datalen = 0, dataalloc = 0, rsrcalloc = 0;
            if (ret.fileattr & ATTR_FILE_LINKCOUNT) { memcpy(&linkcount, q, 4); q += 4; }
            if (ret.fileattr & ATTR_FILE_ALLOCSIZE) { memcpy(&alloc, q, 8); q += 8; }
            if (ret.fileattr & ATTR_FILE_DATALENGTH) { memcpy(&datalen, q, 8); q += 8; }
            if (ret.fileattr & ATTR_FILE_DATAALLOCSIZE) { memcpy(&dataalloc, q, 8); q += 8; }
            if (ret.fileattr & ATTR_FILE_RSRCALLOCSIZE) { memcpy(&rsrcalloc, q, 8); q += 8; }
            int64_t priv = -1; uint64_t cloneid = 0, extflags = 0, gencount = 0; uint32_t refcnt = 0;
            if (ret.forkattr & ATTR_CMNEXT_PRIVATESIZE) { memcpy(&priv, q, 8); q += 8; }
            if (ret.forkattr & ATTR_CMNEXT_CLONEID) { memcpy(&cloneid, q, 8); q += 8; }
            if (ret.forkattr & ATTR_CMNEXT_EXT_FLAGS) { memcpy(&extflags, q, 8); q += 8; }
            if (ret.forkattr & ATTR_CMNEXT_RECURSIVE_GENCOUNT) { memcpy(&gencount, q, 8); q += 8; }
            if (ret.forkattr & ATTR_CMNEXT_CLONE_REFCNT) { memcpy(&refcnt, q, 4); q += 4; }
            p += len;
            n_entries++;
            size_t nl = strlen(name);
            char kind = objtype == VDIR ? 'd' : objtype == VREG ? 'f' : objtype == VLNK ? 'l' : 'o';
            if (kind == 'd') n_dirs++; else if (kind == 'f') { n_files++; tot_alloc += alloc; tot_data += datalen; } else n_other++;
            int pruned = 0;
            if (kind == 'd' && nck && gencount) {
                // relpath = path[rootlen..] + "/" + name
                char rel[8192]; snprintf(rel, sizeof rel, "%s/%s", path + rootlen, name);
                uint64_t g; if (ck_lookup(rel[0] == '/' ? rel + 1 : rel, &g) && g == gencount) { pruned = 1; n_pruned++; }
            }
            if (!quiet && out)
                fprintf(out, "%c\t%s/%s\t%" PRIu64 "\t%u\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%" PRId64 "\t%" PRIu64 "\t0x%" PRIx64 "\t%" PRIu64 "\t%u\t%lld%09ld%s\n",
                        pruned ? 'P' : kind, path, name, fileid, linkcount, datalen, dataalloc, alloc, rsrcalloc, priv, gencount, extflags, cloneid, refcnt, (long long)mt.tv_sec, mt.tv_nsec, err ? "\tERR" : "");
            if (kind == 'd' && !pruned) {
                if (nsub == subcap) { subcap *= 2; subs = realloc(subs, subcap * sizeof *subs); }
                subs[nsub++] = strdup(name);
            }
        }
    }
    for (size_t i = 0; i < nsub; i++) {
        size_t nl = strlen(subs[i]);
        path[plen] = '/'; memcpy(path + plen + 1, subs[i], nl + 1);
        int fd = openat(dfd, subs[i], O_RDONLY | O_DIRECTORY | O_NOFOLLOW);
        if (fd >= 0) { walk(fd, path, plen + 1 + nl); close(fd); }
        else fprintf(stderr, "open %s: %s\n", path, strerror(errno));
        path[plen] = 0;
        free(subs[i]);
    }
    free(subs);
}

static int cmd_walk(int argc, char **argv) {
    const char *root = NULL, *outfile = NULL, *ck = NULL;
    for (int i = 0; i < argc; i++) {
        if (!strcmp(argv[i], "-o") && i + 1 < argc) outfile = argv[++i];
        else if (!strcmp(argv[i], "-p") && i + 1 < argc) ck = argv[++i];
        else if (!strcmp(argv[i], "-q")) quiet = 1;
        else if (!strcmp(argv[i], "-b")) base_only = 1;
        else if (!strcmp(argv[i], "-g")) gen_only = 1;
        else root = argv[i];
    }
    if (!root) return 2;
    if (ck && load_ckpt(ck) != 0) { perror("checkpoint"); return 1; }
    if (outfile) { out = fopen(outfile, "w"); if (!out) { perror(outfile); return 1; } }
    else if (!quiet) out = stdout;
    static char path[16384];
    strlcpy(path, root, sizeof path);
    size_t L = strlen(path); while (L > 1 && path[L - 1] == '/') path[--L] = 0;
    rootlen = L;
    int fd = open(path, O_RDONLY | O_DIRECTORY);
    if (fd < 0) { perror(path); return 1; }
    double t0 = now_ms();
    walk(fd, path, L);
    double t1 = now_ms();
    close(fd);
    if (out && out != stdout) fclose(out);
    printf("SUMMARY entries=%" PRIu64 " dirs=%" PRIu64 " files=%" PRIu64 " other=%" PRIu64 " pruned=%" PRIu64 " wall_ms=%.1f alloc=%" PRIu64 " data=%" PRIu64 "\n",
            n_entries, n_dirs, n_files, n_other, n_pruned, t1 - t0, tot_alloc, tot_data);
    return 0;
}

int main(int argc, char **argv) {
    if (argc < 2) { fprintf(stderr, "usage: ds gen|get|mark|walk ...\n"); return 2; }
    if (!strcmp(argv[1], "gen")) return cmd_gen(argc - 2, argv + 2);
    if (!strcmp(argv[1], "get")) return cmd_get(argc - 2, argv + 2);
    if (!strcmp(argv[1], "mark")) return cmd_mark(argc - 2, argv + 2);
    if (!strcmp(argv[1], "walk")) return cmd_walk(argc - 2, argv + 2);
    fprintf(stderr, "unknown subcommand\n");
    return 2;
}
