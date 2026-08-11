/* Emits the full item/block tables and material predicates for all 256 ids,
   including negative ids, which appear as neighbour-owned border blocks. */
#include <stdio.h>
#include "item.h"

int main(void) {
    printf("# item_count\t%d\n", item_count);
    for (int i = 0; i < item_count; i++) {
        printf("item\t%d\t%d\n", i, items[i]);
    }
    for (int w = 0; w < 256; w++) {
        printf("block\t%d\t%d\t%d\t%d\t%d\t%d\t%d\n", w,
               blocks[w][0], blocks[w][1], blocks[w][2],
               blocks[w][3], blocks[w][4], blocks[w][5]);
    }
    for (int w = 0; w < 256; w++) {
        printf("plant\t%d\t%d\n", w, plants[w]);
    }
    for (int w = 0; w < 256; w++) {
        printf("pred\t%d\t%d\t%d\t%d\t%d\n", w,
               is_plant(w), is_obstacle(w), is_transparent(w), is_destructable(w));
    }
    /* negative ids: border blocks are stored as -w */
    for (int w = -64; w < 0; w++) {
        printf("negpred\t%d\t%d\t%d\t%d\t%d\n", w,
               is_plant(w), is_obstacle(w), is_transparent(w), is_destructable(w));
    }
    return 0;
}
