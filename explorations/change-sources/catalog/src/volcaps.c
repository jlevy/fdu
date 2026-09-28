// volcaps: print ATTR_VOL_CAPABILITIES for each path's volume.
// Usage: volcaps PATH...
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/attr.h>
#include <unistd.h>

struct vol_buf {
    uint32_t length;
    vol_capabilities_attr_t caps;
} __attribute__((packed));

#define FMT(name) { #name, VOL_CAP_FMT_##name, 0 }
#define INT(name) { #name, VOL_CAP_INT_##name, 1 }

static const struct { const char *name; uint32_t bit; int set; } bits[] = {
    FMT(PERSISTENTOBJECTIDS), FMT(SYMBOLICLINKS), FMT(HARDLINKS), FMT(JOURNAL),
    FMT(JOURNAL_ACTIVE), FMT(NO_ROOT_TIMES), FMT(SPARSE_FILES), FMT(ZERO_RUNS),
    FMT(CASE_SENSITIVE), FMT(CASE_PRESERVING), FMT(FAST_STATFS), FMT(2TB_FILESIZE),
    FMT(OPENDENYMODES), FMT(HIDDEN_FILES), FMT(PATH_FROM_ID), FMT(NO_VOLUME_SIZES),
    FMT(DECMPFS_COMPRESSION), FMT(64BIT_OBJECT_IDS), FMT(DIR_HARDLINKS), FMT(DOCUMENT_ID),
    FMT(WRITE_GENERATION_COUNT), FMT(NO_IMMUTABLE_FILES), FMT(NO_PERMISSIONS),
    FMT(SHARED_SPACE), FMT(VOL_GROUPS), FMT(SEALED), FMT(CLONE_MAPPING),
    INT(SEARCHFS), INT(ATTRLIST), INT(NFSEXPORT), INT(READDIRATTR), INT(EXCHANGEDATA),
    INT(COPYFILE), INT(ALLOCATE), INT(VOL_RENAME), INT(ADVLOCK), INT(FLOCK),
    INT(EXTENDED_SECURITY), INT(USERACCESS), INT(MANLOCK), INT(NAMEDSTREAMS),
    INT(EXTENDED_ATTR), INT(CLONE), INT(SNAPSHOT), INT(RENAME_SWAP), INT(RENAME_EXCL),
    INT(RENAME_OPENFAIL), INT(RENAME_SECLUDE),
#ifdef VOL_CAP_INT_ATTRIBUTION_TAG
    INT(ATTRIBUTION_TAG),
#endif
};

int main(int argc, char **argv) {
    for (int i = 1; i < argc; i++) {
        struct attrlist al;
        memset(&al, 0, sizeof al);
        al.bitmapcount = ATTR_BIT_MAP_COUNT;
        al.volattr = ATTR_VOL_INFO | ATTR_VOL_CAPABILITIES;
        struct vol_buf buf;
        memset(&buf, 0, sizeof buf);
        if (getattrlist(argv[i], &al, &buf, sizeof buf, 0) != 0) {
            printf("%s: getattrlist failed: %s\n", argv[i], strerror(errno));
            continue;
        }
        printf("%s: caps fmt=%08x int=%08x valid fmt=%08x int=%08x\n", argv[i],
               buf.caps.capabilities[VOL_CAPABILITIES_FORMAT],
               buf.caps.capabilities[VOL_CAPABILITIES_INTERFACES],
               buf.caps.valid[VOL_CAPABILITIES_FORMAT],
               buf.caps.valid[VOL_CAPABILITIES_INTERFACES]);
        for (size_t b = 0; b < sizeof bits / sizeof bits[0]; b++) {
            uint32_t cap = buf.caps.capabilities[bits[b].set];
            uint32_t valid = buf.caps.valid[bits[b].set];
            printf("  %s_%-24s %s%s\n", bits[b].set ? "INT" : "FMT", bits[b].name,
                   (cap & bits[b].bit) ? "yes" : "no ",
                   (valid & bits[b].bit) ? "" : " (not valid)");
        }
    }
    return 0;
}
