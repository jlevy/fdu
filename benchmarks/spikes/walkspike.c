// walkspike: isolate the cost of Linux directory enumeration + metadata strategies.
// Variants (single-threaded, BFS queue of directory paths):
//   readdir   - opendir/readdir + fstatat(dirfd,name) per entry   (≈ std read_dir + DirEntry::metadata)
//   getdents  - raw getdents64 (256 KiB) + fstatat per entry
//   statx     - raw getdents64 + statx STATX_BASIC_STATS per entry
//   narrow    - raw getdents64 + statx narrow mask (TYPE|SIZE|BLOCKS|MTIME|CTIME|INO)
//   filesonly - raw getdents64 + statx only for DT_REG (summary tier shape; dirs by d_type)
//   inosort   - statx variant, entries statted in ascending d_ino order per directory
//   uring     - raw getdents64 + io_uring-batched statx (QD 128), hand-rolled ring
// Output: dirs, files, other, apparent bytes, allocated bytes, elapsed ns.
// Every variant must report identical dirs/files/bytes on the same immutable tree
// (filesonly reports identical files/bytes; its dir count comes from d_type).
#define _GNU_SOURCE
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <linux/io_uring.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <time.h>
#include <unistd.h>

struct ldirent {
  uint64_t d_ino; int64_t d_off; unsigned short d_reclen; unsigned char d_type; char d_name[];
};

#define SCRATCH (256 * 1024)
static char scratch[SCRATCH];

// simple growable queue of directory paths
static char **queue; static size_t qhead, qtail, qcap;
static void qpush(const char *p) {
  if (qtail == qcap) { qcap = qcap ? qcap * 2 : 1024; queue = realloc(queue, qcap * sizeof(char*)); }
  queue[qtail++] = strdup(p);
}

static uint64_t files, dirs, other, bytes, allocated, stat_calls, enter_calls;

static int64_t now_ns(void) {
  struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
  return (int64_t)ts.tv_sec * 1000000000 + ts.tv_nsec;
}

static void tally(unsigned mode, uint64_t size, uint64_t blocks, const char *path, const char *name) {
  if (S_ISDIR(mode)) { dirs++; char buf[4096]; snprintf(buf, sizeof buf, "%s/%s", path, name); qpush(buf); }
  else if (S_ISREG(mode)) { files++; bytes += size; allocated += blocks * 512; }
  else other++;
}

// ---------- variant: readdir ----------
static void walk_readdir(const char *path) {
  DIR *d = opendir(path); if (!d) return;
  int fd = dirfd(d);
  struct dirent *e;
  while ((e = readdir(d))) {
    if (e->d_name[0] == '.' && (!e->d_name[1] || (e->d_name[1] == '.' && !e->d_name[2]))) continue;
    struct stat sb;
    if (fstatat(fd, e->d_name, &sb, AT_SYMLINK_NOFOLLOW) != 0) continue;
    stat_calls++;
    tally(sb.st_mode, sb.st_size, sb.st_blocks, path, e->d_name);
  }
  closedir(d);
}

// ---------- shared raw-getdents body ----------
typedef void (*stat_fn)(int dirfd, const char *path);

struct pent { uint64_t ino; unsigned char d_type; unsigned short off; }; // offset into namebuf
static char namebuf[SCRATCH * 2];
static struct pent pents[32768];

static int collect(int fd, size_t *count) {
  size_t n = 0, nboff = 0;
  for (;;) {
    ssize_t len = syscall(SYS_getdents64, fd, scratch, SCRATCH);
    if (len <= 0) break;
    for (ssize_t pos = 0; pos < len;) {
      struct ldirent *de = (struct ldirent *)(scratch + pos);
      pos += de->d_reclen;
      if (de->d_name[0] == '.' && (!de->d_name[1] || (de->d_name[1] == '.' && !de->d_name[2]))) continue;
      size_t l = strlen(de->d_name) + 1;
      if (n >= sizeof pents / sizeof *pents || nboff + l > sizeof namebuf) return -1; // too wide, refuse
      memcpy(namebuf + nboff, de->d_name, l);
      pents[n].ino = de->d_ino; pents[n].d_type = de->d_type; pents[n].off = (unsigned short)0;
      // store 32-bit offset in two parts to keep struct tiny: reuse off as index when <=65535, else bail
      if (nboff > 65535) return -1;
      pents[n].off = (unsigned short)nboff;
      nboff += l; n++;
    }
  }
  *count = n;
  return 0;
}

static int cmp_ino(const void *a, const void *b) {
  const struct pent *x = a, *y = b;
  return x->ino < y->ino ? -1 : x->ino > y->ino;
}

static unsigned statx_mask = STATX_BASIC_STATS;

static void statx_one(int fd, const char *name, const char *path) {
  struct statx sx;
  if (statx(fd, name, AT_SYMLINK_NOFOLLOW | AT_NO_AUTOMOUNT, statx_mask, &sx) != 0) return;
  stat_calls++;
  tally(sx.stx_mode, sx.stx_size, sx.stx_blocks, path, name);
}

static void walk_getdents(const char *path, int use_statx, int sort_ino, int files_only) {
  int fd = open(path, O_RDONLY | O_DIRECTORY | O_CLOEXEC | O_NOFOLLOW);
  if (fd < 0) return;
  size_t n;
  if (collect(fd, &n) != 0) { close(fd); walk_readdir(path); return; }
  if (sort_ino) qsort(pents, n, sizeof *pents, cmp_ino);
  for (size_t i = 0; i < n; i++) {
    const char *name = namebuf + pents[i].off;
    if (files_only) {
      unsigned char t = pents[i].d_type;
      if (t == DT_DIR) { dirs++; char buf[4096]; snprintf(buf, sizeof buf, "%s/%s", path, name); qpush(buf); continue; }
      if (t != DT_REG && t != DT_UNKNOWN) { other++; continue; }
      struct statx sx;
      if (statx(fd, name, AT_SYMLINK_NOFOLLOW | AT_NO_AUTOMOUNT, statx_mask, &sx) != 0) continue;
      stat_calls++;
      if (S_ISREG(sx.stx_mode)) { files++; bytes += sx.stx_size; allocated += sx.stx_blocks * 512; }
      else if (S_ISDIR(sx.stx_mode)) { dirs++; char buf[4096]; snprintf(buf, sizeof buf, "%s/%s", path, name); qpush(buf); }
      else other++;
      continue;
    }
    if (use_statx) statx_one(fd, name, path);
    else {
      struct stat sb;
      if (fstatat(fd, name, &sb, AT_SYMLINK_NOFOLLOW) != 0) continue;
      stat_calls++;
      tally(sb.st_mode, sb.st_size, sb.st_blocks, path, name);
    }
  }
  close(fd);
}

// ---------- io_uring statx ----------
#define QD 128
struct uring {
  int fd;
  unsigned *sq_head, *sq_tail, *sq_mask, *sq_array, *cq_head, *cq_tail, *cq_mask;
  struct io_uring_sqe *sqes;
  struct io_uring_cqe *cqes;
};
static struct uring ring;
static struct statx uring_sx[QD];

static int uring_init(void) {
  struct io_uring_params p; memset(&p, 0, sizeof p);
  int fd = (int)syscall(SYS_io_uring_setup, QD, &p);
  if (fd < 0) return -1;
  size_t sq_sz = p.sq_off.array + p.sq_entries * sizeof(unsigned);
  size_t cq_sz = p.cq_off.cqes + p.cq_entries * sizeof(struct io_uring_cqe);
  void *sq = mmap(0, sq_sz, PROT_READ | PROT_WRITE, MAP_SHARED | MAP_POPULATE, fd, IORING_OFF_SQ_RING);
  void *cq = mmap(0, cq_sz, PROT_READ | PROT_WRITE, MAP_SHARED | MAP_POPULATE, fd, IORING_OFF_CQ_RING);
  void *sqes = mmap(0, p.sq_entries * sizeof(struct io_uring_sqe), PROT_READ | PROT_WRITE,
                    MAP_SHARED | MAP_POPULATE, fd, IORING_OFF_SQES);
  if (sq == MAP_FAILED || cq == MAP_FAILED || sqes == MAP_FAILED) return -1;
  ring.fd = fd;
  ring.sq_head = (unsigned *)((char *)sq + p.sq_off.head);
  ring.sq_tail = (unsigned *)((char *)sq + p.sq_off.tail);
  ring.sq_mask = (unsigned *)((char *)sq + p.sq_off.ring_mask);
  ring.sq_array = (unsigned *)((char *)sq + p.sq_off.array);
  ring.cq_head = (unsigned *)((char *)cq + p.cq_off.head);
  ring.cq_tail = (unsigned *)((char *)cq + p.cq_off.tail);
  ring.cq_mask = (unsigned *)((char *)cq + p.cq_off.ring_mask);
  ring.sqes = sqes;
  ring.cqes = (struct io_uring_cqe *)((char *)cq + p.cq_off.cqes);
  return 0;
}

// submit a batch of statx for names[i] (i in [0,n)), harvest results
static void uring_statx_batch(int dirfd, const char *path, size_t start, size_t n) {
  unsigned tail = *ring.sq_tail;
  for (size_t i = 0; i < n; i++) {
    unsigned idx = (tail + i) & *ring.sq_mask;
    struct io_uring_sqe *sqe = &ring.sqes[idx];
    memset(sqe, 0, sizeof *sqe);
    sqe->opcode = IORING_OP_STATX;
    sqe->fd = dirfd;
    sqe->addr = (uint64_t)(namebuf + pents[start + i].off);
    sqe->statx_flags = AT_SYMLINK_NOFOLLOW | AT_NO_AUTOMOUNT;
    sqe->len = statx_mask;
    sqe->off = (uint64_t)&uring_sx[i];
    sqe->user_data = i;
    ring.sq_array[idx] = idx;
  }
  atomic_thread_fence(memory_order_release);
  *ring.sq_tail = tail + n;
  int ret = (int)syscall(SYS_io_uring_enter, ring.fd, n, n, IORING_ENTER_GETEVENTS, NULL, 0);
  enter_calls++;
  if (ret < 0) return;
  unsigned head = *ring.cq_head;
  while (head != *ring.cq_tail) {
    struct io_uring_cqe *cqe = &ring.cqes[head & *ring.cq_mask];
    if (cqe->res == 0) {
      struct statx *sx = &uring_sx[cqe->user_data];
      stat_calls++;
      tally(sx->stx_mode, sx->stx_size, sx->stx_blocks, path, namebuf + pents[start + cqe->user_data].off);
    }
    head++;
  }
  *ring.cq_head = head;
}

static void walk_uring(const char *path) {
  int fd = open(path, O_RDONLY | O_DIRECTORY | O_CLOEXEC | O_NOFOLLOW);
  if (fd < 0) return;
  size_t n;
  if (collect(fd, &n) != 0) { close(fd); walk_readdir(path); return; }
  for (size_t i = 0; i < n; i += QD) {
    size_t batch = n - i < QD ? n - i : QD;
    uring_statx_batch(fd, path, i, batch);
  }
  close(fd);
}

int main(int argc, char **argv) {
  if (argc < 3) { fprintf(stderr, "usage: %s <variant> <root>\n", argv[0]); return 2; }
  const char *variant = argv[1];
  int use_statx = 0, sort_ino = 0, files_only = 0, uring = 0, use_readdir = 0;
  if (!strcmp(variant, "readdir")) use_readdir = 1;
  else if (!strcmp(variant, "getdents")) {}
  else if (!strcmp(variant, "statx")) use_statx = 1;
  else if (!strcmp(variant, "narrow")) { use_statx = 1; statx_mask = STATX_TYPE | STATX_SIZE | STATX_BLOCKS | STATX_MTIME | STATX_CTIME | STATX_INO; }
  else if (!strcmp(variant, "filesonly")) { files_only = 1; statx_mask = STATX_TYPE | STATX_SIZE | STATX_BLOCKS | STATX_MTIME; }
  else if (!strcmp(variant, "inosort")) { use_statx = 1; sort_ino = 1; }
  else if (!strcmp(variant, "uring")) { uring = 1; if (uring_init() != 0) { fprintf(stderr, "io_uring unavailable: %s\n", strerror(errno)); return 3; } }
  else { fprintf(stderr, "unknown variant\n"); return 2; }

  int64_t t0 = now_ns();
  qpush(argv[2]);
  while (qhead < qtail) {
    char *p = queue[qhead++];
    if (use_readdir) walk_readdir(p);
    else if (uring) walk_uring(p);
    else walk_getdents(p, use_statx, sort_ino, files_only);
    free(p);
  }
  int64_t t1 = now_ns();
  printf("{\"variant\":\"%s\",\"dirs\":%lu,\"files\":%lu,\"other\":%lu,\"bytes\":%lu,\"allocated\":%lu,"
         "\"stat_calls\":%lu,\"enter_calls\":%lu,\"wall_ns\":%ld}\n",
         variant, dirs, files, other, bytes, allocated, stat_calls, enter_calls, t1 - t0);
  return 0;
}
