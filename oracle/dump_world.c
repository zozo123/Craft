/* Emits the exact create_world callback sequence for a chunk.
   Order is significant: later emissions overwrite earlier ones (tree leaves
   over terrain), so the Rust port must reproduce the sequence, not just the
   final block set. Negative w marks cells owned by a neighbouring chunk. */
#include <stdio.h>
#include <stdlib.h>
#include "world.h"

static void emit(int x, int y, int z, int w, void *arg) {
    unsigned long *n = (unsigned long *)arg;
    (*n)++;
    printf("%d\t%d\t%d\t%d\n", x, y, z, w);
}

int main(int argc, char **argv) {
    int p = argc > 1 ? atoi(argv[1]) : 0;
    int q = argc > 2 ? atoi(argv[2]) : 0;
    unsigned long n = 0;
    create_world(p, q, emit, &n);
    fprintf(stderr, "chunk %d %d emissions %lu\n", p, q, n);
    return 0;
}
