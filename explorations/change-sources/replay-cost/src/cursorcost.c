// cursorcost: time FSEventsGetLastEventIdForDeviceBeforeTime for a list of ages.
// usage: cursorcost ROOT REPS AGE_SECONDS...
// Prints one JSON line per (age, rep): the id returned and the call's wall time.
#include <CoreServices/CoreServices.h>
#include <inttypes.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/stat.h>
#include <time.h>

static double now(void) { struct timespec t; clock_gettime(CLOCK_MONOTONIC, &t); return t.tv_sec + t.tv_nsec / 1e9; }

int main(int argc, char **argv) {
    if (argc < 4) return 2;
    char root[PATH_MAX]; struct stat st;
    if (!realpath(argv[1], root) || stat(root, &st)) return 2;
    int reps = atoi(argv[2]);
    time_t t_now = time(NULL);
    for (int i = 3; i < argc; i++) {
        long age = atol(argv[i]);
        for (int r = 0; r < reps; r++) {
            double a = now();
            FSEventStreamEventId id = FSEventsGetLastEventIdForDeviceBeforeTime(st.st_dev, (CFAbsoluteTime)(t_now - age));
            double b = now();
            printf("{\"dev\":%d,\"age_s\":%ld,\"rep\":%d,\"wall_now\":%ld,\"id\":%" PRIu64 ",\"ms\":%.3f,\"now_id\":%" PRIu64 "}\n",
                   (int)st.st_dev, age, r, (long)t_now, (uint64_t)id, (b - a) * 1e3, (uint64_t)FSEventsGetCurrentEventId());
        }
    }
    return 0;
}
