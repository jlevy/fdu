// Generic active-writer discovery prototype: libproc only, no root, aggregate output only.
// usage: writers [--regions] [--json] [--verbose-errors]
// Enumerates every pid, lists its descriptors, resolves vnode descriptors with
// PROC_PIDFDVNODEPATHINFO, counts regular files open for write (FWRITE), deleted-but-open
// files (vst_nlink == 0), and optionally walks PROC_PIDREGIONPATHINFO for writable shared
// file-backed mappings. Prints counts, per-volume tallies, error classes, and timings.
// Never prints a path unless --dump-paths-private is given (kept out of any report).
#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <libproc.h>
#include <mach/vm_prot.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mount.h>
#include <sys/proc_info.h>
#include <sys/stat.h>
#include <sys/sysctl.h>
#include <time.h>
#include <unistd.h>

static double monotonic(void) {
    struct timespec value;
    clock_gettime(CLOCK_MONOTONIC, &value);
    return (double)value.tv_sec + (double)value.tv_nsec / 1e9;
}

#define MAX_VOLUMES 64
struct volume { dev_t dev; char mnt[MAXPATHLEN]; char fstype[MFSTYPENAMELEN]; uint64_t write_fds, write_files, deleted_files, deleted_bytes, shared_maps; };
static struct volume volumes[MAX_VOLUMES];
static int nvolumes;

static struct volume *volume_for(dev_t dev) {
    for (int i = 0; i < nvolumes; ++i) if (volumes[i].dev == dev) return &volumes[i];
    if (nvolumes < MAX_VOLUMES) {
        struct volume *v = &volumes[nvolumes++];
        v->dev = dev;
        snprintf(v->mnt, sizeof v->mnt, "dev:%d", (int)dev);
        v->fstype[0] = 0;
        return v;
    }
    return NULL;
}

static void load_mounts(void) {
    struct statfs *mounts;
    int n = getmntinfo(&mounts, MNT_NOWAIT);
    for (int i = 0; i < n; ++i) {
        struct volume *v = volume_for(mounts[i].f_fsid.val[0]);
        if (v) { strlcpy(v->mnt, mounts[i].f_mntonname, sizeof v->mnt); strlcpy(v->fstype, mounts[i].f_fstypename, sizeof v->fstype); }
    }
}

// Unique (dev, ino) set for "files" as opposed to descriptors.
struct key { dev_t dev; uint64_t ino; };
static struct key *seen; static size_t nseen, capseen;
static bool seen_add(dev_t dev, uint64_t ino) {
    for (size_t i = 0; i < nseen; ++i) if (seen[i].dev == dev && seen[i].ino == ino) return false;
    if (nseen == capseen) { capseen = capseen ? capseen * 2 : 1024; seen = realloc(seen, capseen * sizeof *seen); }
    seen[nseen].dev = dev; seen[nseen].ino = ino; ++nseen;
    return true;
}

int main(int argc, char **argv) {
    bool regions = false, regions3 = false, hybrid = false, json = false, verbose_errors = false, dump_private = false;
    for (int i = 1; i < argc; ++i) {
        if (!strcmp(argv[i], "--regions")) regions = true;
        else if (!strcmp(argv[i], "--regions3")) regions3 = true;
        else if (!strcmp(argv[i], "--hybrid")) { regions3 = true; hybrid = true; }
        else if (!strcmp(argv[i], "--json")) json = true;
        else if (!strcmp(argv[i], "--verbose-errors")) verbose_errors = true;
        else if (!strcmp(argv[i], "--dump-paths-private")) dump_private = true;
        else { fprintf(stderr, "usage: writers [--regions] [--json] [--verbose-errors]\n"); return 2; }
    }
    load_mounts();
    uid_t me = getuid();
    double t_start = monotonic();
    int npids = proc_listallpids(NULL, 0);
    if (npids <= 0) { perror("proc_listallpids"); return 1; }
    pid_t *pids = calloc((size_t)npids + 64, sizeof *pids);
    npids = proc_listallpids(pids, (int)(((size_t)npids + 64) * sizeof *pids));
    double t_list = monotonic();

    uint64_t pids_total = 0, pids_ok = 0, pids_eperm = 0, pids_esrch = 0, pids_other = 0, pids_same_uid = 0, pids_same_uid_failed = 0;
    uint64_t fds_total = 0, fds_vnode = 0, fds_vnode_failed = 0, fds_vnode_ebadf = 0, fds_vnode_other = 0;
    uint64_t write_fds = 0, write_regular_files = 0, write_dir_fds = 0, append_fds = 0, deleted_open_fds = 0, deleted_open_files = 0, deleted_open_bytes = 0, deleted_open_alloc = 0;
    uint64_t path_empty = 0, path_maxlen = 0, path_len_max = 0;
    uint64_t pids_with_writers = 0, regions_walked = 0, shared_writable_maps = 0, shared_writable_files = 0, region_pids_failed = 0, pids_region_walked = 0;
    double fd_time = 0, region_time = 0, region_time_max = 0;
    uint64_t region_calls = 0;
    uint64_t vnode_errno_hist[128] = {0};
    uint64_t fds_vnode_nopath_ok = 0, fds_vnode_nopath_failed = 0, write_fds_nopath = 0;
    uint64_t region_errno_hist[128] = {0};
    uint64_t share_hist[8][16] = {{0}};
    uint64_t flags_hist[16] = {0};
    uint64_t hybrid_lookup_failed = 0; // [protection][share_mode] for file-backed regular-file regions

    size_t fdbuf_cap = 4096 * sizeof(struct proc_fdinfo);
    struct proc_fdinfo *fdbuf = malloc(fdbuf_cap);
    for (int i = 0; i < npids; ++i) {
        pid_t pid = pids[i];
        if (pid <= 0) continue;
        ++pids_total;
        struct proc_bsdshortinfo sh;
        bool same_uid = false;
        if (proc_pidinfo(pid, PROC_PIDT_SHORTBSDINFO, 0, &sh, sizeof sh) == (int)sizeof sh) same_uid = sh.pbsi_uid == me;
        if (same_uid) ++pids_same_uid;
        double a = monotonic();
        int size = proc_pidinfo(pid, PROC_PIDLISTFDS, 0, NULL, 0);
        if (size <= 0) {
            int e = errno;
            if (e == EPERM || e == EACCES) ++pids_eperm; else if (e == ESRCH) ++pids_esrch; else ++pids_other;
            if (same_uid) { ++pids_same_uid_failed; if (verbose_errors) fprintf(stderr, "same-uid pid %d listfds failed: %s (pbsi_flags=0x%x)\n", pid, strerror(e), sh.pbsi_flags); }
            continue;
        }
        if ((size_t)size > fdbuf_cap) { fdbuf_cap = (size_t)size * 2; fdbuf = realloc(fdbuf, fdbuf_cap); }
        size = proc_pidinfo(pid, PROC_PIDLISTFDS, 0, fdbuf, (int)fdbuf_cap);
        if (size <= 0) { ++pids_other; continue; }
        ++pids_ok;
        int n = size / (int)sizeof(struct proc_fdinfo);
        bool pid_has_writer = false;
        for (int j = 0; j < n; ++j) {
            ++fds_total;
            if (fdbuf[j].proc_fdtype != PROX_FDTYPE_VNODE) continue;
            ++fds_vnode;
            struct vnode_fdinfowithpath vi;
            int r = proc_pidfdinfo(pid, fdbuf[j].proc_fd, PROC_PIDFDVNODEPATHINFO, &vi, sizeof vi);
            if (r != (int)sizeof vi) {
                ++fds_vnode_failed;
                int e = errno;
                if (e == EBADF) ++fds_vnode_ebadf; else ++fds_vnode_other;
                if (e >= 0 && e < 128) ++vnode_errno_hist[e];
                // Fallback: flags and stat without the path (PROC_PIDFDVNODEINFO) so a denied path
                // still counts as a writer of unknown location.
                struct vnode_fdinfo vi2;
                if (proc_pidfdinfo(pid, fdbuf[j].proc_fd, PROC_PIDFDVNODEINFO, &vi2, sizeof vi2) == (int)sizeof vi2) {
                    ++fds_vnode_nopath_ok;
                    if ((vi2.pfi.fi_openflags & FWRITE) && (vi2.pvi.vi_stat.vst_mode & S_IFMT) == S_IFREG) ++write_fds_nopath;
                } else ++fds_vnode_nopath_failed;
                if (verbose_errors) fprintf(stderr, "pid %d fd %d vnodepathinfo r=%d errno=%d (%s)\n", pid, fdbuf[j].proc_fd, r, e, strerror(e));
                continue;
            }
            if (!(vi.pfi.fi_openflags & FWRITE)) continue;
            if ((vi.pvip.vip_vi.vi_stat.vst_mode & S_IFMT) == S_IFDIR) { ++write_dir_fds; continue; }
            if ((vi.pvip.vip_vi.vi_stat.vst_mode & S_IFMT) != S_IFREG) continue;
            ++write_fds;
            pid_has_writer = true;
            if (vi.pfi.fi_openflags & O_APPEND) ++append_fds;
            size_t plen = strnlen(vi.pvip.vip_path, sizeof vi.pvip.vip_path);
            if (plen == 0) ++path_empty;
            if (plen >= sizeof vi.pvip.vip_path - 1) ++path_maxlen;
            if (plen > path_len_max) path_len_max = plen;
            dev_t dev = vi.pvip.vip_vi.vi_stat.vst_dev;
            struct volume *v = volume_for(dev);
            if (v) ++v->write_fds;
            bool fresh = seen_add(dev, vi.pvip.vip_vi.vi_stat.vst_ino);
            if (fresh) { ++write_regular_files; if (v) ++v->write_files; }
            if (vi.pvip.vip_vi.vi_stat.vst_nlink == 0) {
                ++deleted_open_fds;
                if (fresh) {
                    ++deleted_open_files;
                    deleted_open_bytes += (uint64_t)vi.pvip.vip_vi.vi_stat.vst_size;
                    deleted_open_alloc += (uint64_t)vi.pvip.vip_vi.vi_stat.vst_blocks * 512;
                    if (v) { ++v->deleted_files; v->deleted_bytes += (uint64_t)vi.pvip.vip_vi.vi_stat.vst_blocks * 512; }
                }
            }
            if (dump_private) fprintf(stderr, "pid %d fd %d flags 0x%x nlink %d size %" PRId64 " path %s\n", pid, fdbuf[j].proc_fd, vi.pfi.fi_openflags, vi.pvip.vip_vi.vi_stat.vst_nlink, (int64_t)vi.pvip.vip_vi.vi_stat.vst_size, vi.pvip.vip_path);
        }
        if (pid_has_writer) ++pids_with_writers;
        fd_time += monotonic() - a;
        if (regions3) {
            // Selector 22 (PROC_PIDREGIONPATHINFO2, XNU private header): the kernel skips anonymous regions
            // and returns the next vnode-backed region at or after the address argument.
            double b = monotonic();
            uint64_t address = 0;
            bool any = false;
            for (;;) {
                struct proc_regionwithpathinfo rp;
                ++region_calls;
                int r = proc_pidinfo(pid, 22, address, &rp, sizeof rp);
                if (r != (int)sizeof rp) { if (!any && errno) ++region_errno_hist[errno & 127]; break; }
                any = true;
                ++regions_walked;
                bool writable = rp.prp_prinfo.pri_protection & VM_PROT_WRITE;
                bool shared = rp.prp_prinfo.pri_share_mode == SM_SHARED || rp.prp_prinfo.pri_share_mode == SM_TRUESHARED || rp.prp_prinfo.pri_share_mode == SM_SHARED_ALIASED;
                bool file_backed = rp.prp_vip.vip_path[0] != 0 && (rp.prp_vip.vip_vi.vi_stat.vst_mode & S_IFMT) == S_IFREG;
                if (file_backed) ++share_hist[rp.prp_prinfo.pri_protection & 7][rp.prp_prinfo.pri_share_mode & 15];
                if (file_backed && writable) ++flags_hist[rp.prp_prinfo.pri_flags & 15];
                if (hybrid && file_backed && writable) {
                    // Targeted selector-8 query at this address to recover pri_share_mode.
                    struct proc_regionwithpathinfo rp8;
                    ++region_calls;
                    if (proc_pidinfo(pid, PROC_PIDREGIONPATHINFO, rp.prp_prinfo.pri_address, &rp8, sizeof rp8) == (int)sizeof rp8 && rp8.prp_prinfo.pri_address == rp.prp_prinfo.pri_address) {
                        rp.prp_prinfo.pri_share_mode = rp8.prp_prinfo.pri_share_mode;
                        shared = rp8.prp_prinfo.pri_share_mode == SM_SHARED || rp8.prp_prinfo.pri_share_mode == SM_TRUESHARED || rp8.prp_prinfo.pri_share_mode == SM_SHARED_ALIASED;
                    } else ++hybrid_lookup_failed;
                }
                if (dump_private && file_backed && writable) fprintf(stderr, "pid %d map2-writable prot %d flags 0x%x tag %d path %s\n", pid, rp.prp_prinfo.pri_protection, rp.prp_prinfo.pri_flags, rp.prp_prinfo.pri_user_tag, rp.prp_vip.vip_path);
                if (!hybrid) shared = shared || (rp.prp_prinfo.pri_flags & PROC_REGION_SHARED);
                if (writable && shared && file_backed) {
                    ++shared_writable_maps;
                    struct volume *v = volume_for(rp.prp_vip.vip_vi.vi_stat.vst_dev);
                    if (v) ++v->shared_maps;
                    if (seen_add(rp.prp_vip.vip_vi.vi_stat.vst_dev, rp.prp_vip.vip_vi.vi_stat.vst_ino)) ++shared_writable_files;
                    if (dump_private) fprintf(stderr, "pid %d map2 prot %d share %d flags 0x%x tag %d path %s\n", pid, rp.prp_prinfo.pri_protection, rp.prp_prinfo.pri_share_mode, rp.prp_prinfo.pri_flags, rp.prp_prinfo.pri_user_tag, rp.prp_vip.vip_path);
                }
                uint64_t next = rp.prp_prinfo.pri_address + rp.prp_prinfo.pri_size;
                if (next <= address) break;
                address = next;
            }
            if (any) ++pids_region_walked; else ++region_pids_failed;
            double dt = monotonic() - b;
            region_time += dt;
            if (dt > region_time_max) region_time_max = dt;
        }
        if (regions) {
            double b = monotonic();
            uint64_t address = 0;
            bool any = false;
            for (;;) {
                struct proc_regionwithpathinfo rp;
                ++region_calls;
                int r = proc_pidinfo(pid, PROC_PIDREGIONPATHINFO, address, &rp, sizeof rp);
                if (r != (int)sizeof rp) break;
                any = true;
                ++regions_walked;
                bool writable = rp.prp_prinfo.pri_protection & VM_PROT_WRITE;
                bool shared = rp.prp_prinfo.pri_share_mode == SM_SHARED || rp.prp_prinfo.pri_share_mode == SM_TRUESHARED || rp.prp_prinfo.pri_share_mode == SM_SHARED_ALIASED;
                bool file_backed = rp.prp_vip.vip_path[0] != 0 && (rp.prp_vip.vip_vi.vi_stat.vst_mode & S_IFMT) == S_IFREG;
                if (file_backed) ++share_hist[rp.prp_prinfo.pri_protection & 7][rp.prp_prinfo.pri_share_mode & 15];
                if (writable && shared && file_backed) {
                    ++shared_writable_maps;
                    struct volume *v = volume_for(rp.prp_vip.vip_vi.vi_stat.vst_dev);
                    if (v) ++v->shared_maps;
                    if (seen_add(rp.prp_vip.vip_vi.vi_stat.vst_dev, rp.prp_vip.vip_vi.vi_stat.vst_ino)) ++shared_writable_files;
                    if (dump_private) fprintf(stderr, "pid %d map prot %d share %d flags 0x%x tag %d path %s\n", pid, rp.prp_prinfo.pri_protection, rp.prp_prinfo.pri_share_mode, rp.prp_prinfo.pri_flags, rp.prp_prinfo.pri_user_tag, rp.prp_vip.vip_path);
                }
                uint64_t next = rp.prp_prinfo.pri_address + rp.prp_prinfo.pri_size;
                if (next <= address) break;
                address = next;
            }
            if (any) ++pids_region_walked; else ++region_pids_failed;
            double dt = monotonic() - b;
            region_time += dt;
            if (dt > region_time_max) region_time_max = dt;
        }
    }
    double t_end = monotonic();
    if (json) {
        printf("{\"uid\":%d,\"mode\":\"%s\",\"regions\":%s,\"pids_total\":%" PRIu64 ",\"pids_same_uid\":%" PRIu64 ",\"pids_ok\":%" PRIu64 ",\"pids_eperm\":%" PRIu64 ",\"pids_esrch\":%" PRIu64 ",\"pids_other\":%" PRIu64 ",\"pids_same_uid_failed\":%" PRIu64
               ",\"fds_total\":%" PRIu64 ",\"fds_vnode\":%" PRIu64 ",\"fds_vnode_failed\":%" PRIu64 ",\"fds_vnode_ebadf\":%" PRIu64 ",\"fds_vnode_other\":%" PRIu64
               ",\"write_fds\":%" PRIu64 ",\"write_regular_files\":%" PRIu64 ",\"write_dir_fds\":%" PRIu64 ",\"append_fds\":%" PRIu64 ",\"pids_with_writers\":%" PRIu64
               ",\"deleted_open_fds\":%" PRIu64 ",\"deleted_open_files\":%" PRIu64 ",\"deleted_open_bytes\":%" PRIu64 ",\"deleted_open_alloc\":%" PRIu64
               ",\"write_fds_nopath\":%" PRIu64 ",\"fds_vnode_nopath_ok\":%" PRIu64 ",\"fds_vnode_nopath_failed\":%" PRIu64 ",\"path_empty\":%" PRIu64 ",\"path_maxlen\":%" PRIu64 ",\"path_len_max\":%" PRIu64
               ",\"regions_walked\":%" PRIu64 ",\"region_calls\":%" PRIu64 ",\"pids_region_walked\":%" PRIu64 ",\"region_pids_failed\":%" PRIu64 ",\"shared_writable_maps\":%" PRIu64 ",\"shared_writable_files\":%" PRIu64
               ",\"list_ms\":%.3f,\"fd_ms\":%.3f,\"region_ms\":%.3f,\"region_ms_max_pid\":%.3f,\"total_ms\":%.3f,\"volumes\":[",
               me, hybrid ? "hybrid" : regions3 ? "regions2" : regions ? "regions" : "fd", (regions || regions3) ? "true" : "false", pids_total, pids_same_uid, pids_ok, pids_eperm, pids_esrch, pids_other, pids_same_uid_failed,
               fds_total, fds_vnode, fds_vnode_failed, fds_vnode_ebadf, fds_vnode_other,
               write_fds, write_regular_files, write_dir_fds, append_fds, pids_with_writers,
               deleted_open_fds, deleted_open_files, deleted_open_bytes, deleted_open_alloc,
               write_fds_nopath, fds_vnode_nopath_ok, fds_vnode_nopath_failed, path_empty, path_maxlen, path_len_max,
               regions_walked, region_calls, pids_region_walked, region_pids_failed, shared_writable_maps, shared_writable_files,
               (t_list - t_start) * 1000, fd_time * 1000, region_time * 1000, region_time_max * 1000, (t_end - t_start) * 1000);
        bool first = true;
        for (int i = 0; i < nvolumes; ++i) {
            struct volume *v = &volumes[i];
            if (!v->write_fds && !v->shared_maps) continue;
            printf("%s{\"mount\":\"%s\",\"fstype\":\"%s\",\"write_fds\":%" PRIu64 ",\"write_files\":%" PRIu64 ",\"deleted_files\":%" PRIu64 ",\"deleted_alloc_bytes\":%" PRIu64 ",\"shared_writable_maps\":%" PRIu64 "}",
                   first ? "" : ",", v->mnt, v->fstype, v->write_fds, v->write_files, v->deleted_files, v->deleted_bytes, v->shared_maps);
            first = false;
        }
        printf("]}\n");
    } else {
        printf("pids %" PRIu64 " (same uid %" PRIu64 ", ok %" PRIu64 ", EPERM %" PRIu64 ", ESRCH %" PRIu64 ", other %" PRIu64 ", same-uid failed %" PRIu64 ")\n",
               pids_total, pids_same_uid, pids_ok, pids_eperm, pids_esrch, pids_other, pids_same_uid_failed);
        printf("fds %" PRIu64 " vnode %" PRIu64 " (failed %" PRIu64 ": EBADF %" PRIu64 " other %" PRIu64 ")\n", fds_total, fds_vnode, fds_vnode_failed, fds_vnode_ebadf, fds_vnode_other);
        printf("write fds %" PRIu64 " on %" PRIu64 " regular files in %" PRIu64 " pids; O_APPEND %" PRIu64 "; dir write fds %" PRIu64 "\n", write_fds, write_regular_files, pids_with_writers, append_fds, write_dir_fds);
        printf("deleted-but-open: %" PRIu64 " fds, %" PRIu64 " files, %" PRIu64 " apparent bytes, %" PRIu64 " allocated bytes\n", deleted_open_fds, deleted_open_files, deleted_open_bytes, deleted_open_alloc);
        for (int e = 0; e < 128; ++e) if (vnode_errno_hist[e]) printf("  vnode fdinfo errno %d (%s): %" PRIu64 "\n", e, strerror(e), vnode_errno_hist[e]);
        printf("path-denied fallback (PROC_PIDFDVNODEINFO): ok %" PRIu64 ", failed %" PRIu64 ", of which write fds on regular files %" PRIu64 "\n", fds_vnode_nopath_ok, fds_vnode_nopath_failed, write_fds_nopath);
        printf("paths: empty %" PRIu64 ", at MAXPATHLEN %" PRIu64 ", longest %" PRIu64 "\n", path_empty, path_maxlen, path_len_max);
        for (int e = 0; e < 128; ++e) if (region_errno_hist[e]) printf("  region walk first-call errno %d (%s): %" PRIu64 "\n", e, strerror(e), region_errno_hist[e]);
        if (regions || regions3) { printf("  file-backed region histogram [prot -> share_mode:count] (SM_COW=1 PRIVATE=2 EMPTY=3 SHARED=4 TRUESHARED=5 PRIVATE_ALIASED=6 SHARED_ALIASED=7 LARGE_PAGE=8):\n"); for (int pr = 0; pr < 8; ++pr) { bool anyp = false; for (int sm = 0; sm < 16; ++sm) if (share_hist[pr][sm]) anyp = true; if (!anyp) continue; printf("    prot %d:", pr); for (int sm = 0; sm < 16; ++sm) if (share_hist[pr][sm]) printf(" %d:%" PRIu64, sm, share_hist[pr][sm]); printf("\n"); } }
        if (hybrid) printf("  hybrid: selector-8 lookups that failed or moved: %" PRIu64 "\n", hybrid_lookup_failed);
        if (regions3) { printf("  writable file-backed regions by pri_flags (PROC_REGION_SHARED=1 SUBMAP=2):"); for (int f = 0; f < 16; ++f) if (flags_hist[f]) printf(" 0x%x:%" PRIu64, f, flags_hist[f]); printf("\n"); }
        if (regions || regions3) printf("regions: %" PRIu64 " walked in %" PRIu64 " pids (%" PRIu64 " pids unreadable); writable shared file-backed maps %" PRIu64 " on %" PRIu64 " files\n", regions_walked, pids_region_walked, region_pids_failed, shared_writable_maps, shared_writable_files);
        printf("time: list %.1f ms, fd walk %.1f ms, region walk %.1f ms (max one pid %.1f ms), total %.1f ms\n", (t_list - t_start) * 1000, fd_time * 1000, region_time * 1000, region_time_max * 1000, (t_end - t_start) * 1000);
        for (int i = 0; i < nvolumes; ++i) {
            struct volume *v = &volumes[i];
            if (!v->write_fds && !v->shared_maps) continue;
            printf("  volume %-28s %-6s write fds %4" PRIu64 " files %4" PRIu64 " deleted %3" PRIu64 " (%" PRIu64 " B alloc) shared maps %" PRIu64 "\n", v->mnt, v->fstype, v->write_fds, v->write_files, v->deleted_files, v->deleted_bytes, v->shared_maps);
        }
    }
    free(pids); free(fdbuf); free(seen);
    return 0;
}
