// writers_under: libproc open-writer enumeration restricted to one root, per-file records.
//
// Derived from $REVIEW/writer-coverage/src/writers.c (same enumeration: every pid,
// PROC_PIDLISTFDS, PROC_PIDFDVNODEPATHINFO, FWRITE regular files; optional hybrid
// writable shared-mapping walk). This variant emits one TSV record per (pid, fd) or
// (pid, mapping) whose path is at or under ROOT, so the soak can match writers against
// the watcher's view. Output is PRIVATE: it contains paths. Keep it under private/.
//
// usage: writers_under ROOT [--maps] > records.tsv
// record: ts_ns\tkind\tpid\tfd\tflags\tdev\tino\tnlink\tsize\tblocks\tmtime_ns\tpath
//   kind: fd (write descriptor) | map (writable shared file-backed mapping)
//   fd is -1 for map records; flags is fi_openflags for fd, pri_protection for map.
// A trailer line "#summary ..." carries aggregate counts and timings (safe to publish).
#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <libproc.h>
#include <mach/vm_prot.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/proc_info.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

static double monotonic(void) {
    struct timespec value;
    clock_gettime(CLOCK_MONOTONIC, &value);
    return (double)value.tv_sec + (double)value.tv_nsec / 1e9;
}

static int64_t realtime_ns(void) {
    struct timespec value;
    clock_gettime(CLOCK_REALTIME, &value);
    return (int64_t)value.tv_sec * 1000000000LL + value.tv_nsec;
}

struct key { dev_t dev; uint64_t ino; };
static struct key *seen; static size_t nseen, capseen;
static bool seen_add(dev_t dev, uint64_t ino) {
    for (size_t i = 0; i < nseen; ++i) if (seen[i].dev == dev && seen[i].ino == ino) return false;
    if (nseen == capseen) { capseen = capseen ? capseen * 2 : 1024; seen = realloc(seen, capseen * sizeof *seen); }
    seen[nseen].dev = dev; seen[nseen].ino = ino; ++nseen;
    return true;
}

static bool under_root(const char *path, const char *root, size_t rootlen) {
    if (strncmp(path, root, rootlen) != 0) return false;
    return path[rootlen] == '/' || path[rootlen] == 0;
}

int main(int argc, char **argv) {
    if (argc < 2) { fprintf(stderr, "usage: writers_under ROOT [--maps]\n"); return 2; }
    const char *root = argv[1];
    size_t rootlen = strlen(root);
    while (rootlen > 1 && root[rootlen - 1] == '/') --rootlen;
    bool maps = argc > 2 && !strcmp(argv[2], "--maps");
    uid_t me = getuid();
    int64_t ts = realtime_ns();
    double t_start = monotonic();
    int npids = proc_listallpids(NULL, 0);
    if (npids <= 0) { perror("proc_listallpids"); return 1; }
    pid_t *pids = calloc((size_t)npids + 64, sizeof *pids);
    npids = proc_listallpids(pids, (int)(((size_t)npids + 64) * sizeof *pids));

    uint64_t pids_total = 0, pids_ok = 0, pids_denied = 0, pids_same_uid = 0;
    uint64_t fds_vnode = 0, fds_vnode_failed = 0, write_fds_all = 0, write_files_all = 0;
    uint64_t under_fds = 0, under_files = 0, under_deleted_files = 0, under_deleted_alloc = 0, under_pids = 0;
    uint64_t under_map_files = 0, map_calls = 0;
    double fd_time = 0, map_time = 0;

    size_t fdbuf_cap = 4096 * sizeof(struct proc_fdinfo);
    struct proc_fdinfo *fdbuf = malloc(fdbuf_cap);
    for (int i = 0; i < npids; ++i) {
        pid_t pid = pids[i];
        if (pid <= 0) continue;
        ++pids_total;
        struct proc_bsdshortinfo sh;
        if (proc_pidinfo(pid, PROC_PIDT_SHORTBSDINFO, 0, &sh, sizeof sh) == (int)sizeof sh && sh.pbsi_uid == me) ++pids_same_uid;
        double a = monotonic();
        int size = proc_pidinfo(pid, PROC_PIDLISTFDS, 0, NULL, 0);
        if (size <= 0) { ++pids_denied; continue; }
        if ((size_t)size > fdbuf_cap) { fdbuf_cap = (size_t)size * 2; fdbuf = realloc(fdbuf, fdbuf_cap); }
        size = proc_pidinfo(pid, PROC_PIDLISTFDS, 0, fdbuf, (int)fdbuf_cap);
        if (size <= 0) { ++pids_denied; continue; }
        ++pids_ok;
        int n = size / (int)sizeof(struct proc_fdinfo);
        bool pid_under = false;
        for (int j = 0; j < n; ++j) {
            if (fdbuf[j].proc_fdtype != PROX_FDTYPE_VNODE) continue;
            ++fds_vnode;
            struct vnode_fdinfowithpath vi;
            if (proc_pidfdinfo(pid, fdbuf[j].proc_fd, PROC_PIDFDVNODEPATHINFO, &vi, sizeof vi) != (int)sizeof vi) { ++fds_vnode_failed; continue; }
            if (!(vi.pfi.fi_openflags & FWRITE)) continue;
            if ((vi.pvip.vip_vi.vi_stat.vst_mode & S_IFMT) != S_IFREG) continue;
            ++write_fds_all;
            dev_t dev = vi.pvip.vip_vi.vi_stat.vst_dev;
            uint64_t ino = vi.pvip.vip_vi.vi_stat.vst_ino;
            bool fresh = seen_add(dev, ino);
            if (fresh) ++write_files_all;
            if (!under_root(vi.pvip.vip_path, root, rootlen)) continue;
            ++under_fds;
            pid_under = true;
            if (fresh) {
                ++under_files;
                if (vi.pvip.vip_vi.vi_stat.vst_nlink == 0) { ++under_deleted_files; under_deleted_alloc += (uint64_t)vi.pvip.vip_vi.vi_stat.vst_blocks * 512; }
            }
            printf("%" PRId64 "\tfd\t%d\t%d\t0x%x\t%d\t%" PRIu64 "\t%d\t%" PRId64 "\t%" PRId64 "\t%" PRId64 "\t%s\n",
                   ts, pid, fdbuf[j].proc_fd, vi.pfi.fi_openflags, (int)dev, ino, vi.pvip.vip_vi.vi_stat.vst_nlink,
                   (int64_t)vi.pvip.vip_vi.vi_stat.vst_size, (int64_t)vi.pvip.vip_vi.vi_stat.vst_blocks,
                   (int64_t)vi.pvip.vip_vi.vi_stat.vst_mtime * 1000000000LL + vi.pvip.vip_vi.vi_stat.vst_mtimensec, vi.pvip.vip_path);
        }
        if (pid_under) ++under_pids;
        fd_time += monotonic() - a;
        if (maps) {
            double b = monotonic();
            uint64_t address = 0;
            for (;;) {
                struct proc_regionwithpathinfo rp;
                ++map_calls;
                int r = proc_pidinfo(pid, 22, address, &rp, sizeof rp);
                if (r != (int)sizeof rp) break;
                bool writable = rp.prp_prinfo.pri_protection & VM_PROT_WRITE;
                bool file_backed = rp.prp_vip.vip_path[0] != 0 && (rp.prp_vip.vip_vi.vi_stat.vst_mode & S_IFMT) == S_IFREG;
                if (writable && file_backed && under_root(rp.prp_vip.vip_path, root, rootlen)) {
                    struct proc_regionwithpathinfo rp8;
                    ++map_calls;
                    bool shared = false;
                    if (proc_pidinfo(pid, PROC_PIDREGIONPATHINFO, rp.prp_prinfo.pri_address, &rp8, sizeof rp8) == (int)sizeof rp8 && rp8.prp_prinfo.pri_address == rp.prp_prinfo.pri_address) {
                        int sm = rp8.prp_prinfo.pri_share_mode;
                        shared = sm == SM_SHARED || sm == SM_TRUESHARED || sm == SM_SHARED_ALIASED;
                    }
                    if (shared) {
                        dev_t dev = rp.prp_vip.vip_vi.vi_stat.vst_dev;
                        uint64_t ino = rp.prp_vip.vip_vi.vi_stat.vst_ino;
                        if (seen_add(dev, ino)) ++under_map_files;
                        printf("%" PRId64 "\tmap\t%d\t-1\t0x%x\t%d\t%" PRIu64 "\t%d\t%" PRId64 "\t%" PRId64 "\t%" PRId64 "\t%s\n",
                               ts, pid, rp.prp_prinfo.pri_protection, (int)dev, ino, rp.prp_vip.vip_vi.vi_stat.vst_nlink,
                               (int64_t)rp.prp_vip.vip_vi.vi_stat.vst_size, (int64_t)rp.prp_vip.vip_vi.vi_stat.vst_blocks,
                               (int64_t)rp.prp_vip.vip_vi.vi_stat.vst_mtime * 1000000000LL + rp.prp_vip.vip_vi.vi_stat.vst_mtimensec, rp.prp_vip.vip_path);
                    }
                }
                uint64_t next = rp.prp_prinfo.pri_address + rp.prp_prinfo.pri_size;
                if (next <= address) break;
                address = next;
            }
            map_time += monotonic() - b;
        }
    }
    double t_end = monotonic();
    printf("#summary ts_ns=%" PRId64 " pids=%" PRIu64 " same_uid=%" PRIu64 " ok=%" PRIu64 " denied=%" PRIu64
           " vnode_fds=%" PRIu64 " vnode_failed=%" PRIu64 " write_fds_all=%" PRIu64 " write_files_all=%" PRIu64
           " under_fds=%" PRIu64 " under_files=%" PRIu64 " under_pids=%" PRIu64 " under_deleted_files=%" PRIu64 " under_deleted_alloc=%" PRIu64
           " under_map_files=%" PRIu64 " map_calls=%" PRIu64 " fd_ms=%.1f map_ms=%.1f total_ms=%.1f\n",
           ts, pids_total, pids_same_uid, pids_ok, pids_denied, fds_vnode, fds_vnode_failed, write_fds_all, write_files_all,
           under_fds, under_files, under_pids, under_deleted_files, under_deleted_alloc, under_map_files, map_calls,
           fd_time * 1000, map_time * 1000, (t_end - t_start) * 1000);
    free(pids); free(fdbuf); free(seen);
    return 0;
}
