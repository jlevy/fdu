// searchfs_ct: volume-wide searchfs(2) query by a change-time or modification-time range.
//
// Usage: searchfs_ct VOLPATH {ctime|mtime} LOWER_EPOCH UPPER_EPOCH [options]
//   --max N          maxmatches per call (default 100000)
//   --bufmb M        return buffer size in MiB (default 16)
//   --files-only     SRCHFS_MATCHFILES only (default: files and dirs)
//   --dirs-only      SRCHFS_MATCHDIRS only
//   --timelimit S    per-call time limit seconds (default 120)
//   --dump FILE      write TSV: fileid parentid objtype datalen alloc ctime mtime name
//   --resolve        fsgetpath() each distinct parent id; report resolution cost/failures
//   --subtree PATH   with --resolve: count matches whose parent path is PATH or below
//   --label TEXT     echoed in the summary line
//   --restart-limit N  give up after N EBUSY restarts (default 20)
//
// Summary is one line of key=value pairs on stdout. Records are only written when
// --dump is given (never use --dump on real trees: names are private).
#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/attr.h>
#include <sys/fsgetpath.h>
#include <sys/mount.h>
#include <sys/param.h>
#include <sys/resource.h>
#include <sys/time.h>
#include <sys/vnode.h>
#include <time.h>
#include <unistd.h>

struct time_param {
    uint32_t length;
    int64_t tv_sec;
    int64_t tv_nsec;
} __attribute__((packed));

// Returned record: fixed fields in attribute bit order.
// commonattr = NAME | OBJTYPE | MODTIME | CHGTIME | FILEID | PARENTID
// fileattr   = ALLOCSIZE | DATALENGTH (present for non-directories only, by record layout)
struct rec_fixed {
    uint32_t length;
    attrreference_t name;   // 8
    uint32_t objtype;       // 4
    int64_t mtime_sec, mtime_nsec;  // 16
    int64_t ctime_sec, ctime_nsec;  // 16
    uint64_t fileid;        // 8
    uint64_t parentid;      // 8
} __attribute__((packed));      // 64 bytes

struct rec_file_tail {
    int64_t allocsize;
    int64_t datalength;
} __attribute__((packed));

static double now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1000.0 + ts.tv_nsec / 1e6;
}

static double tv_ms(struct timeval tv) { return tv.tv_sec * 1000.0 + tv.tv_usec / 1000.0; }

// Simple open-addressing hash for parent id -> path (resolution cache).
struct pcache_entry { uint64_t id; char *path; int failed; };
static struct pcache_entry *pcache;
static size_t pcache_cap, pcache_n;

static struct pcache_entry *pcache_lookup(uint64_t id) {
    if (pcache_cap == 0) {
        pcache_cap = 1 << 16;
        pcache = calloc(pcache_cap, sizeof *pcache);
    }
    if (pcache_n * 2 >= pcache_cap) {
        size_t ncap = pcache_cap * 2;
        struct pcache_entry *np = calloc(ncap, sizeof *np);
        for (size_t i = 0; i < pcache_cap; i++) {
            if (pcache[i].id) {
                size_t h = (pcache[i].id * 0x9E3779B97F4A7C15ULL) & (ncap - 1);
                while (np[h].id) h = (h + 1) & (ncap - 1);
                np[h] = pcache[i];
            }
        }
        free(pcache);
        pcache = np;
        pcache_cap = ncap;
    }
    size_t h = (id * 0x9E3779B97F4A7C15ULL) & (pcache_cap - 1);
    while (pcache[h].id && pcache[h].id != id) h = (h + 1) & (pcache_cap - 1);
    if (!pcache[h].id) {
        pcache[h].id = id;
        pcache_n++;
        return &pcache[h];  // path NULL means not yet resolved
    }
    return &pcache[h];
}

static int path_under(const char *path, const char *root, size_t rootlen) {
    if (strncmp(path, root, rootlen) != 0) return 0;
    return path[rootlen] == '\0' || path[rootlen] == '/';
}

int main(int argc, char **argv) {
    if (argc < 5) {
        fprintf(stderr, "usage: %s VOLPATH {ctime|mtime} LOWER UPPER [--max N] [--bufmb M] [--files-only|--dirs-only] [--timelimit S] [--dump FILE] [--resolve] [--subtree PATH] [--label TEXT]\n", argv[0]);
        return 2;
    }
    const char *volpath = argv[1];
    const char *attrname = argv[2];
    long long lower = atoll(argv[3]);
    long long upper = atoll(argv[4]);
    unsigned long maxmatches = 100000;
    size_t bufsize = 16u << 20;
    unsigned int match = SRCHFS_MATCHFILES | SRCHFS_MATCHDIRS;
    long timelimit_s = 120;
    const char *dumpfile = NULL;
    int resolve = 0;
    const char *subtree = NULL;
    const char *label = "";
    int restart_limit = 20;
    for (int i = 5; i < argc; i++) {
        if (!strcmp(argv[i], "--max") && i + 1 < argc) maxmatches = strtoul(argv[++i], NULL, 10);
        else if (!strcmp(argv[i], "--bufmb") && i + 1 < argc) bufsize = (size_t)atoi(argv[++i]) << 20;
        else if (!strcmp(argv[i], "--files-only")) match = SRCHFS_MATCHFILES;
        else if (!strcmp(argv[i], "--dirs-only")) match = SRCHFS_MATCHDIRS;
        else if (!strcmp(argv[i], "--timelimit") && i + 1 < argc) timelimit_s = atol(argv[++i]);
        else if (!strcmp(argv[i], "--dump") && i + 1 < argc) dumpfile = argv[++i];
        else if (!strcmp(argv[i], "--resolve")) resolve = 1;
        else if (!strcmp(argv[i], "--subtree") && i + 1 < argc) { subtree = argv[++i]; resolve = 1; }
        else if (!strcmp(argv[i], "--label") && i + 1 < argc) label = argv[++i];
        else if (!strcmp(argv[i], "--restart-limit") && i + 1 < argc) restart_limit = atoi(argv[++i]);
        else { fprintf(stderr, "unknown option %s\n", argv[i]); return 2; }
    }

    attrgroup_t searchattr;
    if (!strcmp(attrname, "ctime")) searchattr = ATTR_CMN_CHGTIME;
    else if (!strcmp(attrname, "mtime")) searchattr = ATTR_CMN_MODTIME;
    else if (!strcmp(attrname, "crtime")) searchattr = ATTR_CMN_CRTIME;
    else { fprintf(stderr, "attr must be ctime|mtime|crtime\n"); return 2; }

    struct statfs sfs;
    if (statfs(volpath, &sfs) != 0) { perror("statfs"); return 1; }
    fsid_t fsid = sfs.f_fsid;

    FILE *dump = NULL;
    if (dumpfile) {
        dump = fopen(dumpfile, "w");
        if (!dump) { perror("fopen dump"); return 1; }
        fprintf(dump, "fileid\tparentid\tobjtype\tdatalen\talloc\tctime\tmtime\tname\n");
    }

    struct time_param lo = { sizeof(struct time_param), lower, 0 };
    struct time_param hi = { sizeof(struct time_param), upper, 999999999 };

    struct attrlist returnattrs;
    memset(&returnattrs, 0, sizeof returnattrs);
    returnattrs.bitmapcount = ATTR_BIT_MAP_COUNT;
    returnattrs.commonattr = ATTR_CMN_NAME | ATTR_CMN_OBJTYPE | ATTR_CMN_MODTIME | ATTR_CMN_CHGTIME |
                             ATTR_CMN_FILEID | ATTR_CMN_PARENTID;
    returnattrs.fileattr = ATTR_FILE_ALLOCSIZE | ATTR_FILE_DATALENGTH;

    char *buf = malloc(bufsize);
    if (!buf) { perror("malloc"); return 1; }

    struct fssearchblock sb;
    memset(&sb, 0, sizeof sb);
    sb.returnattrs = &returnattrs;
    sb.returnbuffer = buf;
    sb.returnbuffersize = bufsize;
    sb.maxmatches = maxmatches;
    sb.timelimit.tv_sec = timelimit_s;
    sb.timelimit.tv_usec = 0;
    sb.searchparams1 = &lo;
    sb.sizeofsearchparams1 = sizeof lo;
    sb.searchparams2 = &hi;
    sb.sizeofsearchparams2 = sizeof hi;
    sb.searchattrs.bitmapcount = ATTR_BIT_MAP_COUNT;
    sb.searchattrs.commonattr = searchattr;

    struct searchstate state;
    memset(&state, 0, sizeof state);

    struct rusage ru0, ru1;
    getrusage(RUSAGE_SELF, &ru0);
    double t0 = now_ms();

    unsigned long total = 0, files = 0, dirs = 0, others = 0, calls = 0, ebusy = 0;
    unsigned long long sum_datalen = 0, sum_alloc = 0;
    unsigned long parse_errors = 0, name_errors = 0;
    unsigned long min_ct = 0, max_ct = 0; int first = 1;
    unsigned int options = SRCHFS_START | match;
    int err;
    double search_ms = 0;

restart:
    for (;;) {
        unsigned long nummatches = 0;
        double c0 = now_ms();
        int rc = searchfs(volpath, &sb, &nummatches, 0x08000103, options, &state);
        err = rc == 0 ? 0 : errno;
        search_ms += now_ms() - c0;
        calls++;
        if (err == EBUSY) {
            ebusy++;
            if ((int)ebusy > restart_limit) { fprintf(stderr, "too many EBUSY restarts\n"); break; }
            // Restart from scratch: discard everything collected so far.
            total = files = dirs = others = 0; sum_datalen = sum_alloc = 0; first = 1;
            if (dump) { rewind(dump); if (ftruncate(fileno(dump), 0)) {} fprintf(dump, "fileid\tparentid\tobjtype\tdatalen\talloc\tctime\tmtime\tname\n"); }
            options = SRCHFS_START | match;
            memset(&state, 0, sizeof state);
            goto restart;
        }
        if (err != 0 && err != EAGAIN) {
            fprintf(stderr, "searchfs failed: %s (errno %d) after %lu calls\n", strerror(err), err, calls);
            break;
        }
        options &= ~SRCHFS_START;
        char *p = buf;
        for (unsigned long i = 0; i < nummatches; i++) {
            struct rec_fixed *r = (struct rec_fixed *)p;
            if (r->length < sizeof *r || p + r->length > buf + bufsize) { parse_errors++; break; }
            char *name = (char *)&r->name + r->name.attr_dataoffset;
            if (name < p + sizeof *r || name + r->name.attr_length > p + r->length) { name_errors++; name = "?"; }
            int has_file = ((char *)&r->name + r->name.attr_dataoffset) >= p + sizeof *r + sizeof(struct rec_file_tail);
            int64_t alloc = 0, datalen = 0;
            if (has_file) {
                struct rec_file_tail *t = (struct rec_file_tail *)(p + sizeof *r);
                alloc = t->allocsize;
                datalen = t->datalength;
            }
            total++;
            if (r->objtype == VDIR) dirs++;
            else if (r->objtype == VREG) files++;
            else others++;
            sum_datalen += (unsigned long long)datalen;
            sum_alloc += (unsigned long long)alloc;
            unsigned long ct = (unsigned long)r->ctime_sec;
            if (first) { min_ct = max_ct = ct; first = 0; }
            if (ct < min_ct) min_ct = ct;
            if (ct > max_ct) max_ct = ct;
            if (resolve) pcache_lookup(r->parentid);
            if (dump) {
                fprintf(dump, "%" PRIu64 "\t%" PRIu64 "\t%u\t%" PRId64 "\t%" PRId64 "\t%" PRId64 ".%09" PRId64 "\t%" PRId64 ".%09" PRId64 "\t%s\n",
                        r->fileid, r->parentid, r->objtype, datalen, alloc, r->ctime_sec, r->ctime_nsec,
                        r->mtime_sec, r->mtime_nsec, name);
            }
            p += r->length;
        }
        if (err == 0) break;
    }
    double t1 = now_ms();
    getrusage(RUSAGE_SELF, &ru1);

    // Resolution pass.
    unsigned long resolved = 0, resolve_fail = 0, in_subtree_parents = 0;
    double resolve_ms = 0;
    size_t rootlen = subtree ? strlen(subtree) : 0;
    if (resolve) {
        double r0 = now_ms();
        char pathbuf[MAXPATHLEN];
        for (size_t i = 0; i < pcache_cap; i++) {
            if (!pcache[i].id) continue;
            ssize_t n = fsgetpath(pathbuf, sizeof pathbuf, &fsid, pcache[i].id);
            if (n < 0) { pcache[i].failed = errno; resolve_fail++; continue; }
            pcache[i].path = strdup(pathbuf);
            resolved++;
            if (subtree && path_under(pathbuf, subtree, rootlen)) in_subtree_parents++;
        }
        resolve_ms = now_ms() - r0;
    }
    if (dump) fclose(dump);

    printf("label=%s vol=%s attr=%s lower=%lld upper=%lld window_s=%lld matches=%lu files=%lu dirs=%lu others=%lu "
           "sum_datalen=%llu sum_alloc=%llu calls=%lu ebusy_restarts=%lu final_err=%d parse_errors=%lu name_errors=%lu "
           "ctime_min=%lu ctime_max=%lu wall_ms=%.1f search_ms=%.1f user_ms=%.1f sys_ms=%.1f maxrss_kb=%ld "
           "distinct_parents=%zu resolved=%lu resolve_fail=%lu resolve_ms=%.1f subtree_parents=%lu\n",
           label, volpath, attrname, lower, upper, upper - lower, total, files, dirs, others, sum_datalen, sum_alloc,
           calls, ebusy, err, parse_errors, name_errors, min_ct, max_ct, t1 - t0, search_ms,
           tv_ms(ru1.ru_utime) - tv_ms(ru0.ru_utime), tv_ms(ru1.ru_stime) - tv_ms(ru0.ru_stime), ru1.ru_maxrss / 1024,
           pcache_n, resolved, resolve_fail, resolve_ms, in_subtree_parents);
    return (err == 0) ? 0 : 1;
}
