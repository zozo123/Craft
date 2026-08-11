/* Emits matrix results as exact hex floats. set_matrix_3d is the highest-risk
   entry: Craft uses a column-major layout and an unusual pitch axis, so the
   Rust port must match these numbers before any glam convenience helper is
   substituted. */
#include <stdio.h>
#include "matrix.h"

static void dump(const char *tag, float *m) {
    printf("%s", tag);
    for (int i = 0; i < 16; i++) {
        printf("\t%a", m[i]);
    }
    printf("\n");
}

int main(void) {
    float m[16], a[16], b[16];

    mat_identity(m);
    dump("identity", m);

    mat_translate(m, 1.5f, -2.25f, 3.75f);
    dump("translate", m);

    mat_rotate(m, 0.0f, 1.0f, 0.0f, 0.7853981634f);
    dump("rotate_y_45", m);

    mat_rotate(m, 1.0f, 0.0f, 0.0f, 0.5f);
    dump("rotate_x_0p5", m);

    mat_rotate(a, 0.0f, 1.0f, 0.0f, 0.3f);
    mat_rotate(b, 1.0f, 0.0f, 0.0f, 0.2f);
    mat_multiply(m, a, b);
    dump("multiply", m);

    mat_frustum(m, -1.0f, 1.0f, -0.75f, 0.75f, 0.125f, 512.0f);
    dump("frustum", m);

    mat_perspective(m, 65.0f, 1.3333333f, 0.125f, 512.0f);
    dump("perspective", m);

    mat_ortho(m, -10.0f, 10.0f, -7.5f, 7.5f, -1.0f, 1.0f);
    dump("ortho", m);

    set_matrix_2d(m, 1024, 768);
    dump("matrix_2d", m);

    set_matrix_item(m, 1024, 768, 2);
    dump("matrix_item", m);

    set_matrix_3d(m, 1024, 768, 1.0f, 18.0f, 3.0f, 0.5f, 0.25f, 65.0f, 0, 10);
    dump("matrix_3d", m);

    set_matrix_3d(m, 1024, 768, 1.0f, 18.0f, 3.0f, 0.5f, 0.25f, 65.0f, 64, 10);
    dump("matrix_3d_ortho", m);

    set_matrix_3d(m, 800, 600, -12.5f, 33.25f, 7.125f, -1.75f, 0.6f, 45.0f, 0, 24);
    dump("matrix_3d_alt", m);

    float planes[6][4];
    set_matrix_3d(m, 1024, 768, 1.0f, 18.0f, 3.0f, 0.5f, 0.25f, 65.0f, 0, 10);
    frustum_planes(planes, 10, m);
    for (int i = 0; i < 6; i++) {
        printf("plane_%d\t%a\t%a\t%a\t%a\n", i,
               planes[i][0], planes[i][1], planes[i][2], planes[i][3]);
    }

    float x = 3.0f, y = 4.0f, z = 12.0f;
    normalize(&x, &y, &z);
    printf("normalize\t%a\t%a\t%a\n", x, y, z);

    return 0;
}
