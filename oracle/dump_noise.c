/* Emits simplex2/simplex3 sample vectors as exact hex floats (%a) so the
   Rust port can be compared bit-for-bit rather than through decimal rounding.
   Sample points deliberately include the exact coordinate scales world.c uses
   (0.01, 0.1, 0.05) plus the raw integer coords used for tree placement. */
#include <stdio.h>
#include "noise.h"

static void s2(const char *tag, float x, float y,
               int octaves, float persistence, float lacunarity) {
    float v = simplex2(x, y, octaves, persistence, lacunarity);
    printf("simplex2\t%s\t%a\t%a\t%d\t%a\t%a\t%a\n",
           tag, x, y, octaves, persistence, lacunarity, v);
}

static void s3(const char *tag, float x, float y, float z,
               int octaves, float persistence, float lacunarity) {
    float v = simplex3(x, y, z, octaves, persistence, lacunarity);
    printf("simplex3\t%s\t%a\t%a\t%a\t%d\t%a\t%a\t%a\n",
           tag, x, y, z, octaves, persistence, lacunarity, v);
}

int main(void) {
    /* world.c terrain: f and g */
    for (int x = -40; x <= 40; x += 7) {
        for (int z = -40; z <= 40; z += 11) {
            s2("terrain_f", x * 0.01f, z * 0.01f, 4, 0.5f, 2.0f);
            s2("terrain_g", -x * 0.01f, -z * 0.01f, 2, 0.9f, 2.0f);
            s2("grass", -x * 0.1f, z * 0.1f, 4, 0.8f, 2.0f);
            s2("flower", x * 0.05f, -z * 0.05f, 4, 0.8f, 2.0f);
            s2("flowertype", x * 0.1f, z * 0.1f, 4, 0.8f, 2.0f);
            s2("tree", (float)x, (float)z, 6, 0.5f, 2.0f);
        }
    }
    /* world.c clouds */
    for (int x = -20; x <= 20; x += 13) {
        for (int y = 64; y < 72; y += 3) {
            for (int z = -20; z <= 20; z += 17) {
                s3("cloud", x * 0.01f, y * 0.1f, z * 0.01f, 8, 0.5f, 2.0f);
            }
        }
    }
    /* main.c plant rotation */
    for (int x = -8; x <= 8; x += 3) {
        for (int z = -8; z <= 8; z += 5) {
            s2("plantrot", (float)x, (float)z, 4, 0.5f, 2.0f);
        }
    }
    /* degenerate and boundary inputs */
    s2("zero", 0.0f, 0.0f, 1, 0.5f, 2.0f);
    s2("one_octave", 0.5f, 0.5f, 1, 1.0f, 1.0f);
    s3("zero3", 0.0f, 0.0f, 0.0f, 1, 0.5f, 2.0f);
    return 0;
}
