#include <gdk-pixbuf/gdk-pixbuf.h>
#include <libtcc.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <math.h>


static TCCState *tcc_state = NULL;
static double (*eval_func)(double, double) = NULL;


void set_expression(const char *expr)
{
    /* Remove any previously compiled function */
    if (tcc_state) {
        tcc_delete(tcc_state);
        tcc_state = NULL;
        eval_func = NULL;
    }
    if (!expr) return;

    /* Generate the C source for a function f(x,y) */
    char source[1024];
    int len = snprintf(source, sizeof(source),
                       "#include <math.h>\n"
                       "double f(double x, double y) { return (%s); }",
                       expr);
    if (len < 0 || len >= (int)sizeof(source)) {
        fprintf(stderr, "Expression too long.\n");
        return;
    }

    /* Create a TCC compilation context */
    tcc_state = tcc_new();
    if (!tcc_state) {
        fprintf(stderr, "Could not create TCC state.\n");
        return;
    }


    tcc_set_output_type(tcc_state, TCC_OUTPUT_MEMORY);


    tcc_add_library_path(tcc_state, "/usr/lib");
    tcc_add_library(tcc_state, "m");


    if (tcc_compile_string(tcc_state, source) == -1) {
        fprintf(stderr, "Compilation failed for expression: %s\n", expr);
        tcc_delete(tcc_state);
        tcc_state = NULL;
        return;
    }


    if (tcc_relocate(tcc_state) < 0) {
        fprintf(stderr, "Relocation failed.\n");
        tcc_delete(tcc_state);
        tcc_state = NULL;
        return;
    }


    eval_func = (double (*)(double, double)) tcc_get_symbol(tcc_state, "f");
    if (!eval_func) {
        fprintf(stderr, "Could not find symbol 'f'.\n");
        tcc_delete(tcc_state);
        tcc_state = NULL;
        return;
    }
}

GdkPixbuf *
create_pixbuf(int width, int height, double rightx, double leftx, double topy, double bottomy)
{
    GdkPixbuf *pixbuf = gdk_pixbuf_new(GDK_COLORSPACE_RGB, TRUE, 8, width, height);

    gdk_pixbuf_fill(pixbuf, 0xffffffff);

    if (!eval_func)
        return pixbuf;

    int n_channels = gdk_pixbuf_get_n_channels(pixbuf);
    int rowstride  = gdk_pixbuf_get_rowstride(pixbuf);
    guchar *pixels = gdk_pixbuf_get_pixels(pixbuf);

    double x_step = (rightx - leftx) / width;
    double y_step = (topy - bottomy) / height;
    const double threshold = 1e-7;   // almost 0

    /* Allocate array for grid point evaluations (corners) */
    int grid_w = width + 1;
    int grid_h = height + 1;
    double *values = malloc(grid_w * grid_h * sizeof(double));
    if (!values) {
        /* On allocation failure just return the white image */
        return pixbuf;
    }

    /* Evaluate the function at every grid point */
    for (int j = 0; j < grid_h; j++) {
        double y = topy - j * y_step;
        for (int i = 0; i < grid_w; i++) {
            double x = leftx + i * x_step;
            values[j * grid_w + i] = eval_func(x, y);
        }
    }

    /* Walk through each 2×2 cell and colour the corresponding pixel */
    for (int j = 0; j < height; j++) {
        guchar *row = pixels + j * rowstride;
        for (int i = 0; i < width; i++) {
            /* Four corners of the cell (i,j) */
            double v00 = values[ j      * grid_w + i    ];
            double v10 = values[ j      * grid_w + i + 1];
            double v01 = values[(j + 1) * grid_w + i    ];
            double v11 = values[(j + 1) * grid_w + i + 1];

            /* Gather only defined (non‑NaN) corners */
            double def[4];
            int ndef = 0;
            if (isfinite(v00)) def[ndef++] = v00;
            if (isfinite(v10)) def[ndef++] = v10;
            if (isfinite(v01)) def[ndef++] = v01;
            if (isfinite(v11)) def[ndef++] = v11;

            /* All corners undefined → white (skip, already white) */
            if (ndef == 0)
                continue;

            /* Any defined corner near zero → black */
            int has_zero = 0;
            for (int k = 0; k < ndef; k++) {
                if (fabs(def[k]) < threshold) {
                    has_zero = 1;
                    break;
                }
            }
            if (has_zero) {
                guchar *p = row + i * n_channels;
                p[0] = 0;   /* R */
                p[1] = 0;   /* G */
                p[2] = 0;   /* B */
                p[3] = 255; /* A */
                continue;
            }

            /* Check sign mixture among defined corners */
            int pos = 0, neg = 0;
            for (int k = 0; k < ndef; k++) {
                if (def[k] > 0.0) pos = 1;
                else if (def[k] < 0.0) neg = 1;
            }
            if (pos && neg) {
                /* Mixed signs → black */
                guchar *p = row + i * n_channels;
                p[0] = 0;
                p[1] = 0;
                p[2] = 0;
                p[3] = 255;
            }
            /* All same sign or only zeros already handled → stays white */
        }
    }

    free(values);
    return pixbuf;
}