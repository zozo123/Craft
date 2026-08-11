/* Emits cube/plant/player/wireframe/character/sphere vertex buffers as exact
   hex floats. Covers the AO-dependent triangle diagonal flip, which triggers
   when ao[i][0] + ao[i][3] > ao[i][1] + ao[i][2]. */
#include <stdio.h>
#include <string.h>
#include "cube.h"

static void dump(const char *tag, const float *d, int floats) {
    printf("%s\t%d", tag, floats);
    for (int i = 0; i < floats; i++) {
        printf("\t%a", d[i]);
    }
    printf("\n");
}

int main(void) {
    static float data[65536];
    float ao[6][4], light[6][4];

    /* all faces visible, flat lighting: grass */
    memset(ao, 0, sizeof(ao));
    for (int i = 0; i < 6; i++)
        for (int j = 0; j < 4; j++) light[i][j] = 0.0f;
    memset(data, 0, sizeof(data));
    make_cube(data, ao, light, 1, 1, 1, 1, 1, 1, 0.0f, 0.0f, 0.0f, 0.5f, 1);
    dump("cube_grass_all", data, 6 * 6 * 10);

    /* top face only: wood */
    memset(data, 0, sizeof(data));
    make_cube(data, ao, light, 0, 0, 1, 0, 0, 0, 2.0f, 3.0f, 4.0f, 0.5f, 5);
    dump("cube_wood_top", data, 1 * 6 * 10);

    /* asymmetric AO to force the diagonal flip on face 0 */
    for (int i = 0; i < 6; i++)
        for (int j = 0; j < 4; j++) ao[i][j] = 0.0f;
    ao[0][0] = 0.4f; ao[0][3] = 0.4f; ao[0][1] = 0.0f; ao[0][2] = 0.0f;
    memset(data, 0, sizeof(data));
    make_cube(data, ao, light, 1, 0, 0, 0, 0, 0, 0.0f, 0.0f, 0.0f, 0.5f, 3);
    dump("cube_ao_flip", data, 1 * 6 * 10);

    /* varied light values */
    for (int i = 0; i < 6; i++)
        for (int j = 0; j < 4; j++) light[i][j] = (float)(i * 4 + j) / 32.0f;
    memset(data, 0, sizeof(data));
    make_cube(data, ao, light, 1, 1, 1, 1, 1, 1, -1.0f, 2.0f, -3.0f, 0.5f, 10);
    dump("cube_light_varied", data, 6 * 6 * 10);

    /* plants at several rotations */
    memset(data, 0, sizeof(data));
    make_plant(data, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.5f, 18, 0.0f);
    dump("plant_18_rot0", data, 4 * 6 * 10);

    memset(data, 0, sizeof(data));
    make_plant(data, 0.25f, 0.5f, 1.0f, 2.0f, 3.0f, 0.5f, 23, 45.0f);
    dump("plant_23_rot45", data, 4 * 6 * 10);

    memset(data, 0, sizeof(data));
    make_player(data, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f);
    dump("player_origin", data, 6 * 6 * 10);

    memset(data, 0, sizeof(data));
    make_player(data, 1.5f, 20.0f, -3.5f, 0.75f, -0.25f);
    dump("player_posed", data, 6 * 6 * 10);

    memset(data, 0, sizeof(data));
    make_cube_wireframe(data, 0.0f, 0.0f, 0.0f, 0.52f);
    dump("wireframe", data, 24 * 3);

    memset(data, 0, sizeof(data));
    make_character(data, 100.0f, 50.0f, 12.0f, 24.0f, 'A');
    dump("char_A", data, 6 * 4);

    memset(data, 0, sizeof(data));
    make_character_3d(data, 1.0f, 2.0f, 3.0f, 0.5f, 2, 'Z');
    dump("char3d_Z", data, 6 * 5);

    for (int detail = 0; detail <= 3; detail++) {
        int faces = 8 * (1 << (2 * detail));
        memset(data, 0, sizeof(data));
        make_sphere(data, 1.0f, detail);
        char tag[32];
        snprintf(tag, sizeof(tag), "sphere_d%d", detail);
        dump(tag, data, faces * 3 * 8);
    }

    return 0;
}
