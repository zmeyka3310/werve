#include <gdk-pixbuf/gdk-pixbuf.h>

GdkPixbuf *
create_pixbuf(int width, int height, double rightx, double leftx, double topy, double bottomy)
{
    GdkPixbuf *pixbuf = gdk_pixbuf_new(GDK_COLORSPACE_RGB, TRUE, 8, width, height);

    if (pixbuf)
    {
        gdk_pixbuf_fill(pixbuf, 0xffffffff);
    }

    return pixbuf;
}