# Builds golden-fixture dumpers from the original C sources.
#
# Only the pure-computation translation units are compiled. The GL/GLFW
# headers are satisfied by oracle/stubs, so this builds and runs headless
# with no glew, glfw, X11, curl or sqlite present.
#
# Goldens must be generated on the same platform that runs the Rust parity
# tests: float results are not guaranteed identical across arch/libm.

CC      ?= gcc
ROOT    := ..
CFLAGS  := -O0 -std=c99 -Wall \
           -I$(ROOT)/src -I$(ROOT)/deps/noise -Istubs
LDLIBS  := -lm

OUT     := build
GOLDEN  ?= ../fixtures/golden

PURE_SRC := $(ROOT)/deps/noise/noise.c \
            $(ROOT)/src/world.c \
            $(ROOT)/src/item.c \
            $(ROOT)/src/map.c \
            $(ROOT)/src/matrix.c \
            $(ROOT)/src/cube.c \
            $(ROOT)/src/sign.c \
            $(ROOT)/src/ring.c

DUMPERS := dump_noise dump_world dump_item dump_matrix dump_cube

.PHONY: all golden clean

all: $(addprefix $(OUT)/,$(DUMPERS))

$(OUT):
	mkdir -p $(OUT)

$(OUT)/%: %.c $(PURE_SRC) | $(OUT)
	$(CC) $(CFLAGS) -o $@ $< $(PURE_SRC) $(LDLIBS)

# Chunks chosen to exercise distinct terrain: origin, offset, negative,
# and a far chunk where the noise coordinates are large.
WORLD_CHUNKS := 0,0 1,0 0,1 -1,-1 3,-7 12,34

golden: all
	mkdir -p $(GOLDEN)
	$(OUT)/dump_noise  > $(GOLDEN)/noise.tsv
	$(OUT)/dump_item   > $(GOLDEN)/item.tsv
	$(OUT)/dump_matrix > $(GOLDEN)/matrix.tsv
	$(OUT)/dump_cube   > $(GOLDEN)/cube.tsv
	for pq in $(WORLD_CHUNKS); do \
	  p=$${pq%,*}; q=$${pq#*,}; \
	  $(OUT)/dump_world $$p $$q > $(GOLDEN)/world_$${p}_$${q}.tsv \
	    2> $(GOLDEN)/world_$${p}_$${q}.meta; \
	done
	cd $(GOLDEN) && { command -v sha256sum >/dev/null \
	    && sha256sum *.tsv || shasum -a 256 *.tsv; } > SHA256SUMS
	@echo "--- goldens written to $(GOLDEN) ---"
	@wc -l $(GOLDEN)/*.tsv

clean:
	rm -rf $(OUT)
