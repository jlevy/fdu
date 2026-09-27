// Standalone research instrument. Build with the Apple SDK; no fdu linkage.
#include <CoreServices/CoreServices.h>
#include <dispatch/dispatch.h>
#include <errno.h>
#include <inttypes.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mount.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

enum { MAX_EVENTS = 100000, HISTORY_TIMEOUT_SECONDS = 10 };
static const CFTimeInterval STREAM_LATENCY_SECONDS = 0.01;

struct context {
    dispatch_semaphore_t history;
    const char *relative_root;
    size_t count;
    bool history_done;
    bool truncated;
    FSEventStreamEventId history_id;
};

static void hex(const char *s) {
    putchar('"');
    for (const unsigned char *p = (const unsigned char *)s; *p; ++p)
        printf("%02x", *p);
    putchar('"');
}

static void callback(ConstFSEventStreamRef stream, void *info, size_t count,
                     void *paths, const FSEventStreamEventFlags flags[],
                     const FSEventStreamEventId ids[]) {
    (void)stream;
    struct context *ctx = info;
    const char **names = paths;
    for (size_t i = 0; i < count; ++i) {
        if (ctx->count++ >= MAX_EVENTS) {
            ctx->truncated = true;
            continue;
        }
        bool control = (flags[i] & kFSEventStreamEventFlagHistoryDone) != 0;
        const char *name = names[i];
        while (*name == '/') ++name;
        size_t n = strlen(ctx->relative_root);
        bool inside = strncmp(name, ctx->relative_root, n) == 0 &&
                      (name[n] == '/' || name[n] == '\0');
        size_t name_len = strlen(name);
        while (name_len && name[name_len - 1] == '/') --name_len;
        bool ancestor = !inside && strncmp(name, ctx->relative_root, name_len) == 0 &&
                        ctx->relative_root[name_len] == '/';
        printf("{\"type\":\"event\",\"id\":%" PRIu64 ",\"flags\":%u,"
               "\"inside\":%s,\"ancestor\":%s,\"after_history_done\":%s,\"path_hex\":", ids[i], flags[i],
               inside ? "true" : "false", ancestor ? "true" : "false",
               ctx->history_done ? "true" : "false");
        // Control paths have no meaning; never leak device-relative private paths.
        hex(inside && !control ? name + n + (name[n] == '/') : "");
        puts("}");
        if (control) {
            ctx->history_id = ids[i];
            ctx->history_done = true;
            dispatch_semaphore_signal(ctx->history);
        }
    }
}

static uint64_t number(const char *s) {
    char *end;
    errno = 0;
    unsigned long long value = strtoull(s, &end, 10);
    if (errno || !*s || *s == '-' || *end) {
        fprintf(stderr, "invalid unsigned integer\n");
        exit(2);
    }
    return value;
}

static double monotonic(void) {
    struct timespec value;
    if (clock_gettime(CLOCK_MONOTONIC, &value) != 0) exit(2);
    return (double)value.tv_sec + (double)value.tv_nsec / 1e9;
}

int main(int argc, char **argv) {
    if (argc != 3 && argc != 5) {
        fprintf(stderr, "usage: probe capture ROOT | probe replay ROOT CURSOR FLAGS\n");
        return 2;
    }
    bool capture = strcmp(argv[1], "capture") == 0;
    if ((capture && argc != 3) || (!capture && (strcmp(argv[1], "replay") || argc != 5)))
        return 2;
    char root[PATH_MAX];
    struct stat st;
    struct statfs fs;
    if (!realpath(argv[2], root) || stat(root, &st) || statfs(root, &fs)) {
        perror("fixture root");
        return 2;
    }
    if (!S_ISDIR(st.st_mode)) return 2;
    CFUUIDRef uuid = FSEventsCopyUUIDForDevice(st.st_dev);
    if (!uuid) { fprintf(stderr, "journal UUID unavailable\n"); return 2; }
    CFStringRef uuid_string = CFUUIDCreateString(NULL, uuid);
    char uuid_buf[64];
    if (!uuid_string || !CFStringGetCString(uuid_string, uuid_buf, sizeof(uuid_buf), kCFStringEncodingUTF8))
        return 2;
    CFRelease(uuid_string);
    CFRelease(uuid);
    // This API unusually takes Unix-epoch seconds despite its CFAbsoluteTime type.
    FSEventStreamEventId fence = FSEventsGetLastEventIdForDeviceBeforeTime(st.st_dev, (CFAbsoluteTime)time(NULL));
    printf("{\"type\":\"identity\",\"journal_uuid\":\"%s\",\"device\":%d,"
           "\"filesystem\":\"%s\",\"device_fence\":%" PRIu64 ",\"global_now\":%" PRIu64 "}\n",
           uuid_buf, st.st_dev, fs.f_fstypename, fence, FSEventsGetCurrentEventId());
    if (capture) return fence == 0 ? 2 : 0;
    uint64_t cursor = number(argv[3]);
    uint64_t requested_flags = number(argv[4]);
    const uint64_t allowed = kFSEventStreamCreateFlagFileEvents | kFSEventStreamCreateFlagFullHistory;
    if (cursor == UINT64_MAX || requested_flags & ~allowed) return 2;
    const char *relative = root;
    size_t mount_len = strlen(fs.f_mntonname);
    if (strncmp(root, fs.f_mntonname, mount_len) == 0 &&
        (root[mount_len] == '/' || !root[mount_len])) relative += mount_len;
    while (*relative == '/') ++relative;
    CFStringRef path = CFStringCreateWithCString(NULL, relative, kCFStringEncodingUTF8);
    if (!path) return 2;
    CFArrayRef paths = CFArrayCreate(NULL, (const void **)&path, 1, &kCFTypeArrayCallBacks);
    struct context ctx = {.history = dispatch_semaphore_create(0), .relative_root = relative};
    FSEventStreamContext stream_context = {.info = &ctx};
    FSEventStreamRef stream = FSEventStreamCreateRelativeToDevice(
        NULL, callback, &stream_context, st.st_dev, paths, cursor,
        STREAM_LATENCY_SECONDS, (FSEventStreamCreateFlags)requested_flags);
    CFRelease(paths);
    CFRelease(path);
    if (!stream) return 2;
    dispatch_queue_t queue = dispatch_queue_create("fdu.replay.probe", DISPATCH_QUEUE_SERIAL);
    FSEventStreamSetDispatchQueue(stream, queue);
    double start = monotonic();
    bool started = FSEventStreamStart(stream);
    long timeout = started ? dispatch_semaphore_wait(ctx.history,
        dispatch_time(DISPATCH_TIME_NOW, HISTORY_TIMEOUT_SECONDS * NSEC_PER_SEC)) : 1;
    double history_end = monotonic();
    // The Python parent bounds this synchronous flush and kills a stuck subprocess.
    // It collects buffered contemporary events after the historical sentinel.
    if (started && !timeout) FSEventStreamFlushSync(stream);
    double flush_end = monotonic();
    if (started) FSEventStreamStop(stream);
    FSEventStreamInvalidate(stream);
    // All callback storage stays alive through stop, invalidation, and queue drain.
    dispatch_sync(queue, ^{});
    FSEventStreamEventId latest = FSEventStreamGetLatestEventId(stream);
    FSEventStreamRelease(stream);
    printf("{\"type\":\"summary\",\"pid\":%d,\"cursor\":%" PRIu64
           ",\"create_flags\":%" PRIu64 ",\"started\":%s,\"history_done\":%s,"
           "\"timeout\":%s,\"truncated\":%s,\"latest_id\":%" PRIu64
           ",\"history_id\":%" PRIu64 ",\"history_ms\":%.3f,\"flush_ms\":%.3f,\"elapsed_ms\":%.3f}\n",
           getpid(), cursor, requested_flags, started ? "true" : "false",
           ctx.history_done ? "true" : "false", timeout ? "true" : "false",
           ctx.truncated ? "true" : "false", latest, ctx.history_id,
           (history_end - start) * 1000, (flush_end - history_end) * 1000,
           (monotonic() - start) * 1000);
    dispatch_release(queue);
    dispatch_release(ctx.history);
    return started && !timeout && !ctx.truncated ? 0 : 1;
}
