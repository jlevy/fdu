// parfloor: the *parallel* syscall floor for a metadata walk.
//
// walkspike.c isolates the same layer single-threaded, which answers "which syscall
// strategy is cheapest per entry". It cannot answer "how close is fdu to the machine",
// because fdu is parallel and a single-threaded floor is not its lower bound. This is
// that lower bound: N worker threads over a shared directory queue, raw getdents64 plus
// one statx per entry, four integer accumulators, and nothing else. No index, no paths
// retained, no allocation per entry, no delta contract.
//
// Variants:
//   enum     - openat + getdents64 + close only; no per-entry metadata. The floor for
//              ripgrep's job (find files), not du's.
//   stat     - enum plus statx(dirfd, name) per entry. The floor for du's job, and the
//              denominator fdu's aggregate tier should be read against.
//   abspath  - stat, but each entry is statted by full absolute path instead of
//              dirfd-relative. This is what walkdir and ignore do (both call
//              fs::symlink_metadata(&self.path)); std's DirEntry::metadata, which fdu
//              uses, is dirfd-relative. The only difference is how many path components
//              the kernel resolves per entry, so the gap between `stat` and `abspath`
//              prices that choice on its own.
//
// Output is one JSON line with the same tallies walkspike reports, so any variant that
// disagrees with any other, with walkspike, or with fdu's summary is broken.
//
//   gcc -O2 -pthread -o parfloor parfloor.c
//   ./parfloor stat /path/to/tree 4
#define _GNU_SOURCE
#include <dirent.h>
#include <fcntl.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <time.h>
#include <unistd.h>

struct ldirent {
  uint64_t d_ino;
  int64_t d_off;
  unsigned short d_reclen;
  unsigned char d_type;
  char d_name[];
};

#define BUFSZ (64 * 1024)
#define MAXTHREADS 64

// Shared queue of absolute directory paths. Directories are the unit of work: one open,
// one enumeration, one statx per child. Handing out anything finer would put the queue
// on the critical path of the thing being measured.
static char **queue;
static size_t qhead, qtail, qcap;
static pthread_mutex_t qlock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t qcond = PTHREAD_COND_INITIALIZER;
static int idle, nthreads, finished;

static void qpush_locked(const char *p) {
  if (qtail == qcap) {
    qcap = qcap ? qcap * 2 : 1024;
    queue = realloc(queue, qcap * sizeof *queue);
    if (!queue) {
      fprintf(stderr, "parfloor: out of memory\n");
      exit(1);
    }
  }
  queue[qtail++] = strdup(p);
}

// Returns a directory to walk, or NULL once every worker has run dry at the same time
// — which is the only moment no further work can appear.
static char *qpop(void) {
  pthread_mutex_lock(&qlock);
  for (;;) {
    if (qhead < qtail) {
      char *p = queue[qhead++];
      pthread_mutex_unlock(&qlock);
      return p;
    }
    if (finished) break;
    if (++idle == nthreads) {
      finished = 1;
      idle--;
      pthread_cond_broadcast(&qcond);
      break;
    }
    pthread_cond_wait(&qcond, &qlock);
    idle--;
  }
  pthread_mutex_unlock(&qlock);
  return NULL;
}

static int variant_stat, variant_abs;

struct tally {
  uint64_t dirs, files, other, bytes, allocated;
  uint64_t stat_calls, getdents_calls;
};

static int64_t now_ns(void) {
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return (int64_t)ts.tv_sec * 1000000000 + ts.tv_nsec;
}

static void *worker(void *arg) {
  struct tally *t = arg;
  char *buf = malloc(BUFSZ);
  char abs[4096];
  if (!buf) return NULL;
  char *dir;
  while ((dir = qpop()) != NULL) {
    int fd = open(dir, O_RDONLY | O_DIRECTORY | O_CLOEXEC);
    if (fd < 0) {
      free(dir);
      continue;
    }
    t->dirs++;
    for (;;) {
      long n = syscall(SYS_getdents64, fd, buf, BUFSZ);
      t->getdents_calls++;
      if (n <= 0) break;
      for (long off = 0; off < n;) {
        struct ldirent *e = (struct ldirent *)(buf + off);
        off += e->d_reclen;
        const char *name = e->d_name;
        if (name[0] == '.' && (!name[1] || (name[1] == '.' && !name[2]))) continue;
        // d_type resolves directories without a metadata call. `enum` stops here; that
        // is exactly the work a search tool such as ripgrep has to do and no more.
        if (!variant_stat) {
          if (e->d_type == DT_DIR) {
            snprintf(abs, sizeof abs, "%s/%s", dir, name);
            pthread_mutex_lock(&qlock);
            qpush_locked(abs);
            pthread_cond_signal(&qcond);
            pthread_mutex_unlock(&qlock);
          }
          continue;
        }
        struct statx sx;
        int ok;
        if (variant_abs) {
          snprintf(abs, sizeof abs, "%s/%s", dir, name);
          ok = statx(AT_FDCWD, abs, AT_SYMLINK_NOFOLLOW, STATX_BASIC_STATS, &sx) == 0;
        } else {
          ok = statx(fd, name, AT_SYMLINK_NOFOLLOW, STATX_BASIC_STATS, &sx) == 0;
        }
        if (!ok) continue;
        t->stat_calls++;
        if (S_ISDIR(sx.stx_mode)) {
          snprintf(abs, sizeof abs, "%s/%s", dir, name);
          pthread_mutex_lock(&qlock);
          qpush_locked(abs);
          pthread_cond_signal(&qcond);
          pthread_mutex_unlock(&qlock);
        } else if (S_ISREG(sx.stx_mode)) {
          t->files++;
          t->bytes += sx.stx_size;
          t->allocated += sx.stx_blocks * 512;
        } else {
          t->other++;
        }
      }
    }
    close(fd);
    free(dir);
  }
  free(buf);
  return NULL;
}

int main(int argc, char **argv) {
  if (argc < 3) {
    fprintf(stderr, "usage: %s <enum|stat|abspath> <root> [threads]\n", argv[0]);
    return 2;
  }
  const char *v = argv[1];
  if (!strcmp(v, "enum")) {
    variant_stat = 0;
  } else if (!strcmp(v, "stat")) {
    variant_stat = 1;
  } else if (!strcmp(v, "abspath")) {
    variant_stat = 1;
    variant_abs = 1;
  } else {
    fprintf(stderr, "parfloor: unknown variant %s\n", v);
    return 2;
  }
  nthreads = argc > 3 ? atoi(argv[3]) : 1;
  if (nthreads < 1) nthreads = 1;
  if (nthreads > MAXTHREADS) nthreads = MAXTHREADS;

  qpush_locked(argv[2]);
  struct tally tallies[MAXTHREADS];
  pthread_t ids[MAXTHREADS];
  memset(tallies, 0, sizeof tallies);
  int64_t t0 = now_ns();
  for (int i = 0; i < nthreads; i++) pthread_create(&ids[i], NULL, worker, &tallies[i]);
  for (int i = 0; i < nthreads; i++) pthread_join(ids[i], NULL);
  int64_t wall = now_ns() - t0;

  struct tally s;
  memset(&s, 0, sizeof s);
  for (int i = 0; i < nthreads; i++) {
    s.dirs += tallies[i].dirs;
    s.files += tallies[i].files;
    s.other += tallies[i].other;
    s.bytes += tallies[i].bytes;
    s.allocated += tallies[i].allocated;
    s.stat_calls += tallies[i].stat_calls;
    s.getdents_calls += tallies[i].getdents_calls;
  }
  // walkspike counts the root as a directory it descended but not as an entry; match it
  // so the two instruments' tallies are directly comparable.
  printf("{\"variant\":\"%s\",\"threads\":%d,\"dirs\":%llu,\"files\":%llu,\"other\":%llu,"
         "\"bytes\":%llu,\"allocated\":%llu,\"stat_calls\":%llu,\"getdents_calls\":%llu,"
         "\"wall_ns\":%lld}\n",
         v, nthreads, (unsigned long long)s.dirs - 1, (unsigned long long)s.files,
         (unsigned long long)s.other, (unsigned long long)s.bytes,
         (unsigned long long)s.allocated, (unsigned long long)s.stat_calls,
         (unsigned long long)s.getdents_calls, (long long)wall);
  return 0;
}
