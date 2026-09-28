/*
 * rusage_sampler.c -- sample per-process I/O counters without root on macOS.
 *
 * Uses proc_listallpids(3) + proc_pid_rusage(3) with RUSAGE_INFO_V6 to read
 * ri_diskio_bytesread / ri_diskio_byteswritten / ri_logical_writes for every
 * pid, twice INTERVAL seconds apart, and reports:
 *   - the cost of one full sample (wall clock, plus counts of ESRCH/EPERM/other),
 *   - EPERM rates split by same-uid vs other-uid processes,
 *   - deltas aggregated by process-name category (no private names in stdout;
 *     raw per-pid rows go to the optional --raw FILE for local inspection only).
 *
 * Build: clang -O2 -Wall -o rusage_sampler rusage_sampler.c
 * Usage: rusage_sampler [--interval SEC] [--raw FILE] [--top N]
 *
 * Correctness-only experiment: no writes to any real tree, no timing lock needed.
 */
#include <errno.h>
#include <inttypes.h>
#include <libproc.h>
#include <mach/mach_time.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <sys/resource.h>
#include <sys/sysctl.h>
#include <sys/time.h>
#include <sys/types.h>
#include <unistd.h>

typedef struct {
    pid_t pid;
    uid_t uid;
    bool have_uid;
    int rusage_err;             /* 0 = ok, else errno */
    char name[64];
    struct rusage_info_v6 ri;
} sample_row_t;

typedef struct {
    sample_row_t *rows;
    int n;
    double wall_seconds;
    int n_listed;
    int n_ok, n_esrch, n_eperm, n_other;
    int n_same_uid, n_same_uid_ok, n_same_uid_eperm;
    int n_other_uid, n_other_uid_ok, n_other_uid_eperm;
} sample_t;

static double now_seconds(void) {
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return (double)tv.tv_sec + (double)tv.tv_usec / 1e6;
}

static bool uid_of(pid_t pid, uid_t *uid_out) {
    struct kinfo_proc kp;
    size_t len = sizeof(kp);
    int mib[4] = {CTL_KERN, KERN_PROC, KERN_PROC_PID, pid};
    if (sysctl(mib, 4, &kp, &len, NULL, 0) != 0 || len == 0) return false;
    *uid_out = kp.kp_eproc.e_ucred.cr_uid;
    return true;
}

static void take_sample(sample_t *s) {
    memset(s, 0, sizeof(*s));
    double t0 = now_seconds();
    int bytes = proc_listallpids(NULL, 0);
    if (bytes <= 0) { perror("proc_listallpids"); exit(1); }
    int cap = bytes / (int)sizeof(pid_t) + 64;
    pid_t *pids = calloc((size_t)cap, sizeof(pid_t));
    int n = proc_listallpids(pids, cap * (int)sizeof(pid_t));
    if (n <= 0) { perror("proc_listallpids"); exit(1); }
    s->n_listed = n;
    s->rows = calloc((size_t)n, sizeof(sample_row_t));
    uid_t me = getuid();
    for (int i = 0; i < n; i++) {
        sample_row_t *r = &s->rows[s->n];
        r->pid = pids[i];
        if (r->pid <= 0) continue;
        r->have_uid = uid_of(r->pid, &r->uid);
        int rc = proc_pid_rusage(r->pid, RUSAGE_INFO_V6, (rusage_info_t *)&r->ri);
        r->rusage_err = (rc == 0) ? 0 : errno;
        if (rc == 0) proc_name(r->pid, r->name, sizeof(r->name));
        else proc_name(r->pid, r->name, sizeof(r->name)); /* may still work */
        bool same = r->have_uid && r->uid == me;
        if (same) s->n_same_uid++; else s->n_other_uid++;
        if (rc == 0) {
            s->n_ok++;
            if (same) s->n_same_uid_ok++; else s->n_other_uid_ok++;
        } else if (r->rusage_err == ESRCH) {
            s->n_esrch++;
        } else if (r->rusage_err == EPERM) {
            s->n_eperm++;
            if (same) s->n_same_uid_eperm++; else s->n_other_uid_eperm++;
        } else {
            s->n_other++;
        }
        s->n++;
    }
    free(pids);
    s->wall_seconds = now_seconds() - t0;
}

/* Process-name category mapping. Only categories are printed to stdout. */
static const struct { const char *pat; const char *cat; } cats[] = {
    {"kernel_task", "kernel_task"},
    {"cargo", "rust build toolchain (cargo/rustc/ld)"},
    {"rustc", "rust build toolchain (cargo/rustc/ld)"},
    {"rust-analyzer", "rust build toolchain (cargo/rustc/ld)"},
    {"ld", "rust build toolchain (cargo/rustc/ld)"},
    {"clang", "C toolchain (clang/cc)"},
    {"cc", "C toolchain (clang/cc)"},
    {"claude", "agent CLI (claude/codex/cursor-agent)"},
    {"codex", "agent CLI (claude/codex/cursor-agent)"},
    {"cursor-agent", "agent CLI (claude/codex/cursor-agent)"},
    {"node", "node.js"},
    {"bun", "node.js"},
    {"python", "python tooling (python/uv/ruff)"},
    {"uv", "python tooling (python/uv/ruff)"},
    {"ruff", "python tooling (python/uv/ruff)"},
    {"Google Chrome", "browser"},
    {"Chromium", "browser"},
    {"Safari", "browser"},
    {"firefox", "browser"},
    {"Arc", "browser"},
    {"Cursor", "editor (Cursor/Code/Electron)"},
    {"Code", "editor (Cursor/Code/Electron)"},
    {"Electron", "editor (Cursor/Code/Electron)"},
    {"mds", "Spotlight (mds/mds_stores/mdworker)"},
    {"mdworker", "Spotlight (mds/mds_stores/mdworker)"},
    {"corespotlightd", "Spotlight (mds/mds_stores/mdworker)"},
    {"fseventsd", "fseventsd"},
    {"backupd", "Time Machine (backupd)"},
    {"git", "git"},
    {"fdu", "fdu"},
    {"Terminal", "terminal app"},
    {"iTerm", "terminal app"},
    {"launchd", "launchd"},
    {"WindowServer", "WindowServer"},
    {"logd", "logd"},
    {"syslogd", "logd"},
    {"bird", "iCloud (bird/cloudd)"},
    {"cloudd", "iCloud (bird/cloudd)"},
    {"fileproviderd", "File Provider"},
    {"Docker", "containers (Docker)"},
    {"com.docker", "containers (Docker)"},
    {"Slack", "chat app"},
    {"Messages", "chat app"},
    {"Mail", "mail app"},
    {"Music", "media app"},
    {"Photos", "media app"},
    {"photolibraryd", "media app"},
    {"softwareupdated", "software update"},
    {"nsurlsessiond", "network daemons"},
    {"trustd", "security daemons"},
    {"XProtect", "security daemons"},
    {"rusage_sampler", "this sampler"},
};

static const char *category_of(const char *name) {
    if (name == NULL || name[0] == 0) return "(unnamed / no proc_name)";
    for (size_t i = 0; i < sizeof(cats) / sizeof(cats[0]); i++) {
        size_t pl = strlen(cats[i].pat);
        if (strncasecmp(name, cats[i].pat, pl) == 0 &&
            (name[pl] == 0 || name[pl] == ' ' || name[pl] == '-' || name[pl] == '_' ||
             name[pl] == '.' || (name[pl] >= '0' && name[pl] <= '9'))) {
            return cats[i].cat;
        }
        /* prefix match for long executable names like "Google Chrome Helper" */
        if (pl > 4 && strncasecmp(name, cats[i].pat, pl) == 0) return cats[i].cat;
    }
    return "other";
}

typedef struct {
    const char *cat;
    uint64_t d_bytes_written, d_bytes_read, d_logical_writes;
    uint64_t abs_bytes_written, abs_logical_writes;
    int procs, procs_new, procs_gone;
} agg_t;

static agg_t *agg_find(agg_t *aggs, int *n, const char *cat) {
    for (int i = 0; i < *n; i++) if (strcmp(aggs[i].cat, cat) == 0) return &aggs[i];
    aggs[*n].cat = cat;
    return &aggs[(*n)++];
}

static int cmp_agg_written(const void *a, const void *b) {
    const agg_t *x = a, *y = b;
    if (x->d_bytes_written < y->d_bytes_written) return 1;
    if (x->d_bytes_written > y->d_bytes_written) return -1;
    return 0;
}

static const sample_row_t *find_row(const sample_t *s, pid_t pid) {
    for (int i = 0; i < s->n; i++) if (s->rows[i].pid == pid) return &s->rows[i];
    return NULL;
}

static void print_sample_stats(const char *label, const sample_t *s) {
    printf("%s: listed=%d rows=%d ok=%d ESRCH=%d EPERM=%d other=%d wall=%.1f ms (%.1f us/pid)\n",
           label, s->n_listed, s->n, s->n_ok, s->n_esrch, s->n_eperm, s->n_other,
           s->wall_seconds * 1e3, s->wall_seconds * 1e6 / (s->n > 0 ? s->n : 1));
    printf("  same-uid: %d procs, rusage ok=%d EPERM=%d | other-uid: %d procs, ok=%d EPERM=%d\n",
           s->n_same_uid, s->n_same_uid_ok, s->n_same_uid_eperm,
           s->n_other_uid, s->n_other_uid_ok, s->n_other_uid_eperm);
}

static void fmt_bytes(uint64_t b, char *out, size_t n) {
    const char *u[] = {"B", "KiB", "MiB", "GiB", "TiB"};
    double v = (double)b; int i = 0;
    while (v >= 1024.0 && i < 4) { v /= 1024.0; i++; }
    snprintf(out, n, "%.1f %s", v, u[i]);
}

int main(int argc, char **argv) {
    double interval = 60.0;
    const char *rawpath = NULL;
    int top = 15;
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--interval") == 0 && i + 1 < argc) interval = atof(argv[++i]);
        else if (strcmp(argv[i], "--raw") == 0 && i + 1 < argc) rawpath = argv[++i];
        else if (strcmp(argv[i], "--top") == 0 && i + 1 < argc) top = atoi(argv[++i]);
        else { fprintf(stderr, "usage: %s [--interval SEC] [--raw FILE] [--top N]\n", argv[0]); return 2; }
    }
    printf("rusage_sampler: uid=%d euid=%d flavor=RUSAGE_INFO_V6 interval=%.0fs\n", getuid(), geteuid(), interval);

    /* Warm-up / cost measurement: three back-to-back samples. */
    sample_t w;
    for (int k = 0; k < 3; k++) {
        take_sample(&w);
        printf("warmup sample %d: %d pids in %.1f ms\n", k + 1, w.n, w.wall_seconds * 1e3);
        free(w.rows);
    }

    sample_t s1, s2;
    take_sample(&s1);
    print_sample_stats("sample 1", &s1);
    double t_between0 = now_seconds();
    usleep((useconds_t)(interval * 1e6));
    take_sample(&s2);
    double elapsed = now_seconds() - t_between0;
    print_sample_stats("sample 2", &s2);
    printf("elapsed between samples: %.1f s\n", elapsed);

    FILE *raw = rawpath ? fopen(rawpath, "w") : NULL;
    if (raw) fprintf(raw, "pid\tuid\terr1\terr2\tname\tcategory\tbw1\tbw2\tbr1\tbr2\tlw1\tlw2\tstart_abstime2\n");

    agg_t aggs[256]; int naggs = 0; memset(aggs, 0, sizeof(aggs));
    uint64_t tot_dw = 0, tot_dr = 0, tot_dl = 0, tot_abs_w = 0, tot_abs_l = 0;
    int n_new = 0, n_gone = 0, n_both = 0;
    uint64_t new_abs_w = 0, gone_abs_w = 0;

    for (int i = 0; i < s2.n; i++) {
        const sample_row_t *b = &s2.rows[i];
        if (b->rusage_err != 0) continue;
        const sample_row_t *a = find_row(&s1, b->pid);
        const char *cat = category_of(b->name);
        agg_t *g = agg_find(aggs, &naggs, cat);
        g->procs++;
        g->abs_bytes_written += b->ri.ri_diskio_byteswritten;
        g->abs_logical_writes += b->ri.ri_logical_writes;
        tot_abs_w += b->ri.ri_diskio_byteswritten;
        tot_abs_l += b->ri.ri_logical_writes;
        bool same_incarnation = a && a->rusage_err == 0 &&
            a->ri.ri_proc_start_abstime == b->ri.ri_proc_start_abstime;
        uint64_t dw, dr, dl;
        if (same_incarnation) {
            n_both++;
            dw = b->ri.ri_diskio_byteswritten - a->ri.ri_diskio_byteswritten;
            dr = b->ri.ri_diskio_bytesread - a->ri.ri_diskio_bytesread;
            dl = b->ri.ri_logical_writes - a->ri.ri_logical_writes;
        } else {
            /* process started after sample 1 (or pid reused): whole lifetime counts */
            n_new++; g->procs_new++;
            dw = b->ri.ri_diskio_byteswritten;
            dr = b->ri.ri_diskio_bytesread;
            dl = b->ri.ri_logical_writes;
            new_abs_w += dw;
        }
        g->d_bytes_written += dw; g->d_bytes_read += dr; g->d_logical_writes += dl;
        tot_dw += dw; tot_dr += dr; tot_dl += dl;
        if (raw) fprintf(raw, "%d\t%d\t%d\t%d\t%s\t%s\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\n",
                         b->pid, b->have_uid ? (int)b->uid : -1, a ? a->rusage_err : -1, b->rusage_err,
                         b->name, cat,
                         a && a->rusage_err == 0 ? a->ri.ri_diskio_byteswritten : 0, b->ri.ri_diskio_byteswritten,
                         a && a->rusage_err == 0 ? a->ri.ri_diskio_bytesread : 0, b->ri.ri_diskio_bytesread,
                         a && a->rusage_err == 0 ? a->ri.ri_logical_writes : 0, b->ri.ri_logical_writes,
                         b->ri.ri_proc_start_abstime);
    }
    /* processes present in sample 1 but gone in sample 2: their writes between samples are lost */
    for (int i = 0; i < s1.n; i++) {
        const sample_row_t *a = &s1.rows[i];
        if (a->rusage_err != 0) continue;
        const sample_row_t *b = find_row(&s2, a->pid);
        if (!b || b->rusage_err != 0 || b->ri.ri_proc_start_abstime != a->ri.ri_proc_start_abstime) {
            n_gone++;
            gone_abs_w += a->ri.ri_diskio_byteswritten;
            agg_t *g = agg_find(aggs, &naggs, category_of(a->name));
            g->procs_gone++;
        }
    }
    if (raw) fclose(raw);

    qsort(aggs, (size_t)naggs, sizeof(agg_t), cmp_agg_written);
    char b1[32], b2[32], b3[32];
    printf("\nprocesses: matched=%d new-since-sample-1=%d gone-since-sample-1=%d\n", n_both, n_new, n_gone);
    fmt_bytes(new_abs_w, b1, sizeof b1); fmt_bytes(gone_abs_w, b2, sizeof b2);
    printf("  lifetime bytes-written of new procs (counted in deltas): %s; of gone procs (their post-sample-1 writes are unobservable): %s\n", b1, b2);
    fmt_bytes(tot_dw, b1, sizeof b1); fmt_bytes(tot_dr, b2, sizeof b2); fmt_bytes(tot_dl, b3, sizeof b3);
    printf("totals over %.0f s: diskio_byteswritten +%s, diskio_bytesread +%s, logical_writes +%s\n", elapsed, b1, b2, b3);
    fmt_bytes(tot_abs_w, b1, sizeof b1); fmt_bytes(tot_abs_l, b2, sizeof b2);
    printf("lifetime totals of live procs at sample 2: diskio_byteswritten %s, logical_writes %s\n", b1, b2);
    printf("\ntop categories by diskio_byteswritten delta:\n");
    printf("  %-44s %6s %12s %12s %12s %5s %5s\n", "category", "procs", "written+", "read+", "logical+", "new", "gone");
    for (int i = 0; i < naggs && i < top; i++) {
        fmt_bytes(aggs[i].d_bytes_written, b1, sizeof b1);
        fmt_bytes(aggs[i].d_bytes_read, b2, sizeof b2);
        fmt_bytes(aggs[i].d_logical_writes, b3, sizeof b3);
        printf("  %-44s %6d %12s %12s %12s %5d %5d\n", aggs[i].cat, aggs[i].procs, b1, b2, b3, aggs[i].procs_new, aggs[i].procs_gone);
    }
    free(s1.rows); free(s2.rows);
    return 0;
}
