

# detect operating system
uname_s ?= $(shell uname -s)
os_id := NONE
ifeq ("$(uname_s)","Linux")
	os_id = LINUX
endif
ifeq ("$(uname_s)","linux")
	os_id = LINUX
endif
ifneq ("$(filter GNU hurd,$(uname_s))","")
	os_id = LINUX
endif
ifeq ("$(uname_s)","Darwin")
	os_id = OSX
endif
ifeq ("$(uname_s)","FreeBSD")
	os_id = FREEBSD
endif
ifeq ("$(uname_s)","OpenBSD")
	os_id = FREEBSD
endif
ifneq ("$(filter GNU/kFreeBSD kfreebsd,$(uname_s))","")
	os_id = LINUX
endif
ifeq ("$(patsubst MINGW%,MINGW,$(uname_s))","MINGW")
	os_id = MINGW
endif
ifeq ("$(os_id)","NONE")
	$(error os_id type "$(uname_s)" not supported by M64+ makefiles.)
endif

# setup output dir
TARGET_DIR ?= target

# decide on target properties
libname = $(1)
pluginname = $(1)
ifeq ($(os_id), FREEBSD)
	core_soname = libmupen64plus.so.2.0.0
	libname = lib$(1).so
	pluginname = $(1).so
endif
ifeq ($(os_id), LINUX)
	core_soname = libmupen64plus.so.2.0.0
	libname = lib$(1).so
	pluginname = $(1).so
endif
ifeq ($(os_id), OSX)
	core_soname = libmupen64plus.dylib
	libname = lib$(1).dylib
	pluginname = $(1).dylib
endif
ifeq ($(os_id), MINGW)
	core_soname = mupen64plus.dll
	libname = $(1).dll
	pluginname = $(1).dll
endif

ifeq ($(PRE_BUILD),)
$(info running pre-build step:)
dummy:=$(shell make PRE_BUILD=1 pre-build)
endif

# define targets
mupen64plus-core/projects/unix/$(core_soname):
	@cd mupen64plus-core/projects/unix && make -j4 TAS=1 all
$(TARGET_DIR)/$(call libname,mupen64plus): mupen64plus-core/projects/unix/$(core_soname)
	@cp -fv mupen64plus-core/projects/unix/$(core_soname) $(TARGET_DIR)/$(call libname,mupen64plus)

mupen64plus-video-rice/projects/unix/$(call pluginname,mupen64plus-video-rice):
	@cd mupen64plus-video-rice/projects/unix && make -j4 all
$(TARGET_DIR)/$(call pluginname,mupen64plus-video-rice): mupen64plus-video-rice/projects/unix/$(call pluginname,mupen64plus-video-rice)
	@cp -fv mupen64plus-video-rice/projects/unix/$(call pluginname,mupen64plus-video-rice) $(TARGET_DIR)/

mupen64plus-audio-sdl/projects/unix/$(call pluginname,mupen64plus-audio-sdl):
	@cd mupen64plus-audio-sdl/projects/unix && make -j4 all
$(TARGET_DIR)/$(call pluginname,mupen64plus-audio-sdl): mupen64plus-audio-sdl/projects/unix/$(call pluginname,mupen64plus-audio-sdl)
	@cp -fv mupen64plus-audio-sdl/projects/unix/$(call pluginname,mupen64plus-audio-sdl) $(TARGET_DIR)/

mupen64plus-input-sdl/projects/unix/$(call pluginname,mupen64plus-input-sdl):
	@cd mupen64plus-input-sdl/projects/unix && make -j4 all
$(TARGET_DIR)/$(call pluginname,mupen64plus-input-sdl): mupen64plus-input-sdl/projects/unix/$(call pluginname,mupen64plus-input-sdl)
	@cp -fv mupen64plus-input-sdl/projects/unix/$(call pluginname,mupen64plus-input-sdl) $(TARGET_DIR)/

mupen64plus-rsp-hle/projects/unix/$(call pluginname,mupen64plus-rsp-hle):
	@cd mupen64plus-rsp-hle/projects/unix && make -j4 all
$(TARGET_DIR)/$(call pluginname,mupen64plus-rsp-hle): mupen64plus-rsp-hle/projects/unix/$(call pluginname,mupen64plus-rsp-hle)
	@cp -fv mupen64plus-rsp-hle/projects/unix/$(call pluginname,mupen64plus-rsp-hle) $(TARGET_DIR)/

.DEFAULT_GOAL := all
.PHONY: all pre-build

target_list = \
	$(TARGET_DIR)/$(call libname,mupen64plus) \
	$(TARGET_DIR)/$(call pluginname,mupen64plus-video-rice) \
	$(TARGET_DIR)/$(call pluginname,mupen64plus-audio-sdl) \
	$(TARGET_DIR)/$(call pluginname,mupen64plus-input-sdl) \
	$(TARGET_DIR)/$(call pluginname,mupen64plus-rsp-hle)

$(shell mkdir -p "$(TARGET_DIR)")
all: $(target_list)

pre-build:
	echo "pre-build step!"