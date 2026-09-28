// Pilot: time-to-HistoryDone as a function of cursor age, for a quiet path filter.
#include <CoreServices/CoreServices.h>
#include <dispatch/dispatch.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mount.h>
#include <sys/stat.h>
#include <time.h>
struct ctx { dispatch_semaphore_t done; size_t events; double first; double t0; };
static double now(void){ struct timespec t; clock_gettime(CLOCK_MONOTONIC,&t); return t.tv_sec+t.tv_nsec/1e9; }
static void cb(ConstFSEventStreamRef s, void *info, size_t n, void *paths, const FSEventStreamEventFlags f[], const FSEventStreamEventId ids[]) {
  struct ctx *c = info; if (!c->first) c->first = now() - c->t0;
  for (size_t i=0;i<n;i++){ c->events++; if (f[i] & kFSEventStreamEventFlagHistoryDone) dispatch_semaphore_signal(c->done); }
}
int main(int argc, char **argv) {
  // agesweep ROOT AGE_SECONDS FLAGS TIMEOUT
  if (argc != 5) return 2;
  char root[PATH_MAX]; struct stat st; struct statfs fs;
  if (!realpath(argv[1], root) || stat(root,&st) || statfs(root,&fs)) return 2;
  long age = atol(argv[2]); unsigned flags = (unsigned)atol(argv[3]); long to = atol(argv[4]);
  FSEventStreamEventId cursor = FSEventsGetLastEventIdForDeviceBeforeTime(st.st_dev, (CFAbsoluteTime)(time(NULL) - age));
  const char *rel = root; size_t ml = strlen(fs.f_mntonname);
  if (!strncmp(root, fs.f_mntonname, ml)) rel += ml; while (*rel=='/') rel++;
  CFStringRef p = CFStringCreateWithCString(NULL, rel, kCFStringEncodingUTF8);
  CFArrayRef ps = CFArrayCreate(NULL,(const void**)&p,1,&kCFTypeArrayCallBacks);
  struct ctx c = {.done = dispatch_semaphore_create(0)};
  FSEventStreamContext sc = {.info=&c};
  double t0 = now(); c.t0 = t0;
  FSEventStreamRef s = FSEventStreamCreateRelativeToDevice(NULL, cb, &sc, st.st_dev, ps, cursor, 0.01, flags);
  dispatch_queue_t q = dispatch_queue_create("p", DISPATCH_QUEUE_SERIAL);
  FSEventStreamSetDispatchQueue(s, q);
  double tc = now();
  FSEventStreamStart(s);
  double ts = now();
  long r = dispatch_semaphore_wait(c.done, dispatch_time(DISPATCH_TIME_NOW, to*NSEC_PER_SEC));
  double th = now();
  FSEventStreamStop(s); FSEventStreamInvalidate(s); dispatch_sync(q, ^{}); FSEventStreamRelease(s);
  double te = now();
  printf("{\"fs\":\"%s\",\"age_s\":%ld,\"flags\":%u,\"cursor\":%llu,\"now_id\":%llu,\"create_ms\":%.1f,\"start_ms\":%.1f,\"first_cb_ms\":%.1f,\"history_ms\":%.1f,\"teardown_ms\":%.1f,\"timeout\":%s,\"events\":%zu}\n",
    fs.f_fstypename, age, flags, cursor, FSEventsGetCurrentEventId(), (tc-t0)*1e3, (ts-tc)*1e3, c.first*1e3, (th-ts)*1e3, (te-th)*1e3, r?"true":"false", c.events);
  return r ? 1 : 0;
}
