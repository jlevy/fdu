// Read APFS clone/private-size attributes for one path (getattrlist) or for every
// entry in a directory (getattrlistbulk), and time the bulk call with and without the
// CMNEXT group and ATTR_FILE_LINKCOUNT. Metadata only; never opens file contents.
// Attributes are packed in bitmap order within each group (file: LINKCOUNT, ALLOCSIZE,
// DATALENGTH; cmnext: PRIVATESIZE, CLONEID, EXT_FLAGS, CLONE_REFCNT).
//   privsize file PATH        -- sizes, private size, clone id/refcnt, ext flags
//   privsize path PATH        -- NOFIRMLINKPATH, REALDEVID, REALFSID (root identity)
//   privsize bulk DIR [reps]  -- timing: base / +linkcount / +cmnext
#include <sys/attr.h>
#include <sys/mount.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/vnode.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <stdint.h>
#include <time.h>

static double now(void){ struct timespec t; clock_gettime(CLOCK_MONOTONIC,&t); return t.tv_sec + t.tv_nsec/1e9; }

static int file_mode(const char *path){
    struct attrlist al; memset(&al,0,sizeof al);
    al.bitmapcount = ATTR_BIT_MAP_COUNT;
    al.commonattr = ATTR_CMN_RETURNED_ATTRS;
    al.fileattr = ATTR_FILE_LINKCOUNT | ATTR_FILE_ALLOCSIZE | ATTR_FILE_DATALENGTH;
    al.forkattr = ATTR_CMNEXT_PRIVATESIZE | ATTR_CMNEXT_CLONEID | ATTR_CMNEXT_EXT_FLAGS | ATTR_CMNEXT_CLONE_REFCNT;
    char buf[512]; memset(buf,0,sizeof buf);
    if (getattrlist(path,&al,buf,sizeof buf,FSOPT_ATTR_CMN_EXTENDED|FSOPT_NOFOLLOW)!=0){ perror("getattrlist"); return 1; }
    char *q = buf+4; attribute_set_t ret = *(attribute_set_t*)q; q += sizeof(attribute_set_t);
    uint32_t nlink=0; off_t alloc=0, datalen=0; uint64_t priv=0,cid=0,ef=0; uint32_t refc=0;
    if (ret.fileattr & ATTR_FILE_LINKCOUNT){ nlink=*(uint32_t*)q; q+=4; }
    if (ret.fileattr & ATTR_FILE_ALLOCSIZE){ alloc=*(off_t*)q; q+=8; }
    if (ret.fileattr & ATTR_FILE_DATALENGTH){ datalen=*(off_t*)q; q+=8; }
    if (ret.forkattr & ATTR_CMNEXT_PRIVATESIZE){ priv=*(uint64_t*)q; q+=8; }
    if (ret.forkattr & ATTR_CMNEXT_CLONEID){ cid=*(uint64_t*)q; q+=8; }
    if (ret.forkattr & ATTR_CMNEXT_EXT_FLAGS){ ef=*(uint64_t*)q; q+=8; }
    if (ret.forkattr & ATTR_CMNEXT_CLONE_REFCNT){ refc=*(uint32_t*)q; q+=4; }
    printf("path=%s returned(file=0x%x cmnext=0x%x)\n  nlink=%u datalength=%lld allocsize=%lld privatesize=%llu cloneid=%llu clonerefcnt=%u extflags=0x%llx EF_MAY_SHARE_BLOCKS=%d\n",
        path, ret.fileattr, ret.forkattr, nlink,(long long)datalen,(long long)alloc,(unsigned long long)priv,(unsigned long long)cid,refc,(unsigned long long)ef,(int)((ef & EF_MAY_SHARE_BLOCKS)!=0));
    return 0;
}

static int path_mode(const char *path){
    struct attrlist al; memset(&al,0,sizeof al);
    al.bitmapcount = ATTR_BIT_MAP_COUNT;
    al.commonattr = ATTR_CMN_RETURNED_ATTRS | ATTR_CMN_DEVID | ATTR_CMN_FSID | ATTR_CMN_FILEID;
    al.forkattr = ATTR_CMNEXT_NOFIRMLINKPATH | ATTR_CMNEXT_REALDEVID | ATTR_CMNEXT_REALFSID;
    char buf[4096]; memset(buf,0,sizeof buf);
    if (getattrlist(path,&al,buf,sizeof buf,FSOPT_ATTR_CMN_EXTENDED|FSOPT_NOFOLLOW)!=0){ perror("getattrlist"); return 1; }
    char *q = buf+4; attribute_set_t ret = *(attribute_set_t*)q; q += sizeof(attribute_set_t);
    dev_t devid=0; fsid_t fsid={{0,0}}; uint64_t fileid=0; const char *nofirm="(none)"; dev_t realdev=0; fsid_t realfs={{0,0}};
    if (ret.commonattr & ATTR_CMN_DEVID){ devid=*(dev_t*)q; q+=4; }
    if (ret.commonattr & ATTR_CMN_FSID){ fsid=*(fsid_t*)q; q+=8; }
    if (ret.commonattr & ATTR_CMN_FILEID){ fileid=*(uint64_t*)q; q+=8; }
    if (ret.forkattr & ATTR_CMNEXT_NOFIRMLINKPATH){ attrreference_t *ar=(attrreference_t*)q; nofirm = q + ar->attr_dataoffset; q += sizeof(attrreference_t); }
    if (ret.forkattr & ATTR_CMNEXT_REALDEVID){ realdev=*(dev_t*)q; q+=4; }
    if (ret.forkattr & ATTR_CMNEXT_REALFSID){ realfs=*(fsid_t*)q; q+=8; }
    printf("path=%s returned(common=0x%x cmnext=0x%x)\n  devid=%d fsid=%d,%d fileid=%llu\n  nofirmlinkpath=%s realdevid=%d realfsid=%d,%d\n",
        path, ret.commonattr, ret.forkattr, devid, fsid.val[0], fsid.val[1], (unsigned long long)fileid, nofirm, realdev, realfs.val[0], realfs.val[1]);
    return 0;
}

static int bulk_once(const char *dir, int mode, long *entries, unsigned long long *alloc_sum, unsigned long long *priv_sum, long *multi){
    int fd = open(dir, O_RDONLY|O_DIRECTORY|O_NOFOLLOW);
    if (fd<0){ perror("open"); return 1; }
    struct attrlist al; memset(&al,0,sizeof al);
    al.bitmapcount = ATTR_BIT_MAP_COUNT;
    al.commonattr = ATTR_CMN_RETURNED_ATTRS | ATTR_CMN_NAME | ATTR_CMN_OBJTYPE | ATTR_CMN_ERROR;
    al.fileattr = ATTR_FILE_ALLOCSIZE | ATTR_FILE_DATALENGTH;
    if (mode>=1) al.fileattr |= ATTR_FILE_LINKCOUNT;
    if (mode>=2) al.forkattr = ATTR_CMNEXT_PRIVATESIZE | ATTR_CMNEXT_CLONEID | ATTR_CMNEXT_EXT_FLAGS | ATTR_CMNEXT_CLONE_REFCNT;
    static char buf[256*1024];
    for(;;){
        int n = getattrlistbulk(fd,&al,buf,sizeof buf,FSOPT_ATTR_CMN_EXTENDED|FSOPT_NOFOLLOW);
        if (n<0){ perror("getattrlistbulk"); close(fd); return 1; }
        if (n==0) break;
        char *p = buf;
        for (int i=0;i<n;i++){
            char *rec = p; uint32_t len = *(uint32_t*)rec; char *q = rec+4;
            attribute_set_t ret = *(attribute_set_t*)q; q += sizeof(attribute_set_t);
            if (ret.commonattr & ATTR_CMN_ERROR){ q += 4; }
            if (ret.commonattr & ATTR_CMN_NAME){ q += sizeof(attrreference_t); }
            uint32_t objtype = 0; if (ret.commonattr & ATTR_CMN_OBJTYPE){ objtype = *(uint32_t*)q; q += 4; }
            uint32_t nlink=1; off_t allocsz=0;
            if (ret.fileattr & ATTR_FILE_LINKCOUNT){ nlink=*(uint32_t*)q; q+=4; }
            if (ret.fileattr & ATTR_FILE_ALLOCSIZE){ allocsz=*(off_t*)q; q+=8; }
            if (ret.fileattr & ATTR_FILE_DATALENGTH){ q+=8; }
            uint64_t priv=0;
            if (ret.forkattr & ATTR_CMNEXT_PRIVATESIZE){ priv=*(uint64_t*)q; q+=8; }
            if (objtype==VREG){ (*entries)++; *alloc_sum += (unsigned long long)allocsz; *priv_sum += priv; if (nlink>1) (*multi)++; }
            p = rec + len;
        }
    }
    close(fd); return 0;
}

int main(int argc,char **argv){
    if (argc<3){ fprintf(stderr,"usage: %s file PATH | path PATH | bulk DIR [reps]\n",argv[0]); return 2; }
    if (!strcmp(argv[1],"file")) return file_mode(argv[2]);
    if (!strcmp(argv[1],"path")) return path_mode(argv[2]);
    int reps = argc>3 ? atoi(argv[3]) : 5;
    const char *names[3] = {"base(alloc,datalen)", "+linkcount", "+linkcount+cmnext"};
    for (int m=0; m<3; m++){
        for (int r=0;r<reps;r++){
            long entries=0, multi=0; unsigned long long a=0,pr=0;
            double t0=now(); if (bulk_once(argv[2], m, &entries,&a,&pr,&multi)) return 1; double t1=now();
            printf("bulk %-22s regular=%ld alloc_sum=%llu priv_sum=%llu multilink=%ld time=%.3f ms (%.2f us/entry)\n",
                names[m], entries, a, pr, multi, (t1-t0)*1e3, entries? (t1-t0)*1e6/entries : 0.0);
        }
    }
    return 0;
}
