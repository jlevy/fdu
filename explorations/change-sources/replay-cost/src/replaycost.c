// replaycost: measure the cost of an FSEvents history replay to HistoryDone.
// Standalone research instrument (review of fdu PR #131). No fdu linkage.
//
// usage: replaycost [options]
//   --root PATH         existing path; the device is taken from it and, unless --filter
//                       is given, its device-relative path becomes the stream filter.
//   --filter REL        device-relative filter string; may repeat (one stream, many
//                       paths). "" means the volume root. Need not exist.
//   --age SECONDS       cursor = FSEventsGetLastEventIdForDeviceBeforeTime(dev, now-age)
//   --cursor ID         explicit cursor (overrides --age)
//   --flags N           FSEventStreamCreate flags (default 16 = FileEvents)
//   --timeout S         seconds to wait for HistoryDone (default 150)
//   --latency S         stream latency (default 0.01)
//   --print             per-event JSON line to stdout inside the callback (probe.c style,
//                       line-buffered). Default: count in memory only.
//   --sleep-cb MS       sleep this long inside every callback (backpressure control)
//   --flush MODE        start-sync | start-async | done-sync ; may repeat
//   --abandon-ms N      give up N ms after Start if HistoryDone has not arrived
//   --abandon MODE      clean (Stop/Invalidate/Release then exit) | exit (_exit, no
//                       teardown) | kill (SIGKILL self). Default clean.
//   --label STR         copied into the output record
//
// Output: one JSON object on the last stdout line. Times in ms. fseventsd CPU is
// sampled with `ps -o time=` (centiseconds) just before stream creation and just after
// teardown; proc_pidinfo is not permitted for a non-root caller on this host.
#include <CoreServices/CoreServices.h>
#include <dispatch/dispatch.h>
#include <errno.h>
#include <inttypes.h>
#include <limits.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mount.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

struct ctx {
    dispatch_semaphore_t done;
    double t_start;
    double first_cb, last_cb, done_at;
    size_t callbacks, events, history_done_events, degraded_events;
    size_t bytes_of_paths;
    uint64_t min_id, max_id, history_id;
    bool print;
    unsigned sleep_ms;
    uint64_t events_after_done;
    bool history_done;
};

static double now(void) {
    struct timespec t;
    clock_gettime(CLOCK_MONOTONIC, &t);
    return t.tv_sec + t.tv_nsec / 1e9;
}

static void cb(ConstFSEventStreamRef s, void *info, size_t n, void *paths,
               const FSEventStreamEventFlags f[], const FSEventStreamEventId ids[]) {
    (void)s;
    struct ctx *c = info;
    double t = now();
    if (!c->callbacks) c->first_cb = t;
    c->last_cb = t;
    c->callbacks++;
    const char **names = paths;
    for (size_t i = 0; i < n; i++) {
        c->events++;
        if (c->history_done) c->events_after_done++;
        size_t len = strlen(names[i]);
        c->bytes_of_paths += len;
        if (ids[i] && (!c->min_id || ids[i] < c->min_id)) c->min_id = ids[i];
        if (ids[i] > c->max_id) c->max_id = ids[i];
        if (f[i] & (kFSEventStreamEventFlagMustScanSubDirs | kFSEventStreamEventFlagUserDropped |
                    kFSEventStreamEventFlagKernelDropped | kFSEventStreamEventFlagEventIdsWrapped))
            c->degraded_events++;
        if (c->print)
            // probe.c prints path hex; we print only length to keep private paths out.
            printf("{\"type\":\"event\",\"id\":%" PRIu64 ",\"flags\":%u,\"len\":%zu}\n",
                   (uint64_t)ids[i], (unsigned)f[i], len);
        if (f[i] & kFSEventStreamEventFlagHistoryDone) {
            c->history_done_events++;
            c->history_id = ids[i];
            c->done_at = t;
            c->history_done = true;
            dispatch_semaphore_signal(c->done);
        }
    }
    if (c->sleep_ms) usleep(c->sleep_ms * 1000);
}

static double fseventsd_cpu(void) {
    // returns seconds of CPU per `ps -o time=` for the fseventsd process, or -1
    FILE *p = popen("/bin/ps -o time= -p $(/usr/bin/pgrep -x fseventsd) 2>/dev/null", "r");
    if (!p) return -1;
    char buf[64] = {0};
    if (!fgets(buf, sizeof buf, p)) { pclose(p); return -1; }
    pclose(p);
    // formats: MM:SS.cc or HH:MM:SS.cc (ps uses M:SS.cc; hours fold into minutes)
    int h = 0, m = 0; double sec = 0;
    if (sscanf(buf, "%d:%d:%lf", &h, &m, &sec) == 3) return h * 3600 + m * 60 + sec;
    if (sscanf(buf, "%d:%lf", &m, &sec) == 2) return m * 60 + sec;
    return -1;
}

static uint64_t number(const char *s) {
    char *end; errno = 0;
    unsigned long long v = strtoull(s, &end, 10);
    if (errno || !*s || *end) { fprintf(stderr, "bad number: %s\n", s); exit(2); }
    return v;
}

int main(int argc, char **argv) {
    const char *root = NULL, *label = "";
    const char *filters[64]; int nfilters = 0;
    long age = -1; uint64_t cursor = UINT64_MAX; unsigned flags = 16; long timeout = 150;
    double latency = 0.01; bool print = false; unsigned sleep_ms = 0;
    bool flush_start_sync = false, flush_start_async = false, flush_done_sync = false;
    long abandon_ms = -1; const char *abandon = "clean";
    for (int i = 1; i < argc; i++) {
        const char *a = argv[i];
        const char *v = (i + 1 < argc) ? argv[i + 1] : NULL;
        if (!strcmp(a, "--root") && v) { root = v; i++; }
        else if (!strcmp(a, "--filter") && v) { if (nfilters < 64) filters[nfilters++] = v; i++; }
        else if (!strcmp(a, "--age") && v) { age = atol(v); i++; }
        else if (!strcmp(a, "--cursor") && v) { cursor = number(v); i++; }
        else if (!strcmp(a, "--flags") && v) { flags = (unsigned)number(v); i++; }
        else if (!strcmp(a, "--timeout") && v) { timeout = atol(v); i++; }
        else if (!strcmp(a, "--latency") && v) { latency = atof(v); i++; }
        else if (!strcmp(a, "--print")) print = true;
        else if (!strcmp(a, "--sleep-cb") && v) { sleep_ms = (unsigned)atol(v); i++; }
        else if (!strcmp(a, "--flush") && v) {
            if (!strcmp(v, "start-sync")) flush_start_sync = true;
            else if (!strcmp(v, "start-async")) flush_start_async = true;
            else if (!strcmp(v, "done-sync")) flush_done_sync = true;
            else { fprintf(stderr, "bad flush mode\n"); return 2; }
            i++;
        }
        else if (!strcmp(a, "--abandon-ms") && v) { abandon_ms = atol(v); i++; }
        else if (!strcmp(a, "--abandon") && v) { abandon = v; i++; }
        else if (!strcmp(a, "--label") && v) { label = v; i++; }
        else { fprintf(stderr, "unknown or incomplete option: %s\n", a); return 2; }
    }
    if (!root || (age < 0 && cursor == UINT64_MAX)) { fprintf(stderr, "need --root and --age/--cursor\n"); return 2; }
    setvbuf(stdout, NULL, _IOLBF, 0);
    char rootbuf[PATH_MAX]; struct stat st; struct statfs fs;
    if (!realpath(root, rootbuf) || stat(rootbuf, &st) || statfs(rootbuf, &fs)) { perror("root"); return 2; }
    char relbuf[PATH_MAX];
    const char *rel = rootbuf; size_t ml = strlen(fs.f_mntonname);
    if (!strncmp(rootbuf, fs.f_mntonname, ml) && (rootbuf[ml] == '/' || !rootbuf[ml])) rel += ml;
    while (*rel == '/') rel++;
    strlcpy(relbuf, rel, sizeof relbuf);
    if (!nfilters) filters[nfilters++] = relbuf;

    double t0 = now();
    time_t wall_now = time(NULL);
    if (cursor == UINT64_MAX)
        cursor = FSEventsGetLastEventIdForDeviceBeforeTime(st.st_dev, (CFAbsoluteTime)(wall_now - age));
    double t_cursor = now();
    uint64_t now_id = FSEventsGetCurrentEventId();

    CFMutableArrayRef ps = CFArrayCreateMutable(NULL, 0, &kCFTypeArrayCallBacks);
    for (int i = 0; i < nfilters; i++) {
        CFStringRef p = CFStringCreateWithCString(NULL, filters[i], kCFStringEncodingUTF8);
        CFArrayAppendValue(ps, p); CFRelease(p);
    }
    struct ctx c = {.done = dispatch_semaphore_create(0), .print = print, .sleep_ms = sleep_ms};
    FSEventStreamContext sc = {.info = &c};
    double cpu0 = fseventsd_cpu();
    double t_pre = now();
    FSEventStreamRef s = FSEventStreamCreateRelativeToDevice(NULL, cb, &sc, st.st_dev, ps, cursor, latency, flags);
    double t_created = now();
    if (!s) { fprintf(stderr, "create failed\n"); return 2; }
    dispatch_queue_t q = dispatch_queue_create("replaycost", DISPATCH_QUEUE_SERIAL);
    FSEventStreamSetDispatchQueue(s, q);
    double t_start = now(); c.t_start = t_start;
    bool started = FSEventStreamStart(s);
    double t_started = now();
    uint64_t async_id = 0; double t_flush_start_ms = 0;
    bool done_before_start_flush = false;
    if (started && flush_start_async) { double a = now(); async_id = FSEventStreamFlushAsync(s); t_flush_start_ms = (now() - a) * 1e3; done_before_start_flush = c.history_done; }
    if (started && flush_start_sync) { double a = now(); done_before_start_flush = c.history_done; FSEventStreamFlushSync(s); t_flush_start_ms = (now() - a) * 1e3; }
    bool done_after_start_flush = c.history_done;
    long waited = 1; bool abandoned = false;
    if (started) {
        long wait_ms = abandon_ms >= 0 ? abandon_ms : timeout * 1000;
        waited = dispatch_semaphore_wait(c.done, dispatch_time(DISPATCH_TIME_NOW, wait_ms * NSEC_PER_MSEC));
        if (waited && abandon_ms >= 0) abandoned = true;
    }
    double t_wait_end = now();
    double t_flush_done_ms = -1;
    if (started && !waited && flush_done_sync) { double a = now(); FSEventStreamFlushSync(s); t_flush_done_ms = (now() - a) * 1e3; }
    double t_before_teardown = now();
    if (abandoned && !strcmp(abandon, "kill")) {
        printf("{\"label\":\"%s\",\"abandon\":\"kill\",\"cursor\":%" PRIu64 ",\"events\":%zu,\"elapsed_ms\":%.1f}\n",
               label, cursor, c.events, (t_before_teardown - t_start) * 1e3);
        fflush(stdout);
        kill(getpid(), SIGKILL);
    }
    if (abandoned && !strcmp(abandon, "exit")) {
        printf("{\"label\":\"%s\",\"abandon\":\"exit\",\"cursor\":%" PRIu64 ",\"events\":%zu,\"elapsed_ms\":%.1f}\n",
               label, cursor, c.events, (t_before_teardown - t_start) * 1e3);
        fflush(stdout);
        _exit(3);
    }
    if (started) FSEventStreamStop(s);
    double t_stopped = now();
    FSEventStreamInvalidate(s);
    dispatch_sync(q, ^{});
    uint64_t latest = FSEventStreamGetLatestEventId(s);
    FSEventStreamRelease(s);
    double t_end = now();
    double cpu1 = fseventsd_cpu();
    printf("{\"label\":\"%s\",\"fs\":\"%s\",\"dev\":%d,\"nfilters\":%d,\"filter0_len\":%zu,\"age_s\":%ld,\"wall_now\":%ld,"
           "\"flags\":%u,\"latency\":%.3f,\"cursor\":%" PRIu64 ",\"now_id\":%" PRIu64 ",\"cursor_lookup_ms\":%.2f,"
           "\"create_ms\":%.2f,\"start_ms\":%.2f,\"started\":%s,"
           "\"first_cb_ms\":%.2f,\"last_cb_ms\":%.2f,\"history_ms\":%.2f,\"wait_ms\":%.2f,\"timeout\":%s,\"abandoned\":%s,\"abandon\":\"%s\","
           "\"history_done\":%s,\"history_id\":%" PRIu64 ",\"latest_id\":%" PRIu64 ",\"min_id\":%" PRIu64 ",\"max_id\":%" PRIu64 ","
           "\"callbacks\":%zu,\"events\":%zu,\"history_done_events\":%zu,\"events_after_done\":%" PRIu64 ",\"degraded\":%zu,\"path_bytes\":%zu,"
           "\"flush_start_sync\":%s,\"flush_start_async\":%s,\"async_id\":%" PRIu64 ",\"flush_start_ms\":%.2f,\"done_before_start_flush\":%s,\"done_after_start_flush\":%s,"
           "\"flush_done_sync_ms\":%.2f,\"stop_ms\":%.2f,\"teardown_ms\":%.2f,\"total_ms\":%.2f,"
           "\"fseventsd_cpu_before\":%.2f,\"fseventsd_cpu_after\":%.2f,\"fseventsd_cpu_delta\":%.2f,\"print\":%s,\"sleep_cb_ms\":%u}\n",
           label, fs.f_fstypename, (int)st.st_dev, nfilters, strlen(filters[0]), age, (long)wall_now,
           flags, latency, cursor, now_id, (t_cursor - t0) * 1e3,
           (t_created - t_pre) * 1e3, (t_started - t_start) * 1e3, started ? "true" : "false",
           c.callbacks ? (c.first_cb - t_start) * 1e3 : -1, c.callbacks ? (c.last_cb - t_start) * 1e3 : -1,
           c.history_done ? (c.done_at - t_start) * 1e3 : -1, (t_wait_end - t_started) * 1e3,
           waited ? "true" : "false", abandoned ? "true" : "false", abandon,
           c.history_done ? "true" : "false", c.history_id, latest, c.min_id, c.max_id,
           c.callbacks, c.events, c.history_done_events, c.events_after_done, c.degraded_events, c.bytes_of_paths,
           flush_start_sync ? "true" : "false", flush_start_async ? "true" : "false", async_id, t_flush_start_ms,
           done_before_start_flush ? "true" : "false", done_after_start_flush ? "true" : "false",
           t_flush_done_ms, (t_stopped - t_before_teardown) * 1e3, (t_end - t_stopped) * 1e3, (t_end - t0) * 1e3,
           cpu0, cpu1, (cpu0 >= 0 && cpu1 >= 0) ? cpu1 - cpu0 : -1, print ? "true" : "false", sleep_ms);
    return c.history_done ? 0 : 1;
}
