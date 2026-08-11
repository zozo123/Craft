/* Minimal GL type stubs so the pure-computation Craft sources
   (item.c, matrix.c, cube.c) compile without a real GL/X11 stack.
   Only the typedefs referenced by src/util.h are provided; any attempt
   to call a real GL function will fail to link, which is intended. */
#ifndef _oracle_glew_stub_h_
#define _oracle_glew_stub_h_

typedef unsigned int GLuint;
typedef int GLsizei;
typedef float GLfloat;
typedef unsigned int GLenum;
typedef int GLint;
typedef char GLchar;

#endif
