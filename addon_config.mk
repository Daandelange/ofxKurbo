meta:
	ADDON_NAME = ofxKurbo
	ADDON_DESCRIPTION = OF wrapper for lib Kurbo, providing a set of geometrical functions on cartesian planar Beziér paths or shapes.
	ADDON_AUTHOR = Daan de Lange
	ADDON_TAGS = "Geometry" "Algorithms" "Animation"
	ADDON_URL = https://github.com/daandelange/ofxKurbo

common:
	ADDON_INCLUDES = src/
	ADDON_INCLUDES += libs/kurbo-ffi/include
	ADDON_LDFLAGS = -lpthread -ldl

osx:
	ADDON_LIBS = libs/kurbo-ffi/lib/osx/libkurbo_ffi.dylib
	#ADDON_LIBS = libs/kurbo-ffi/lib/osx/libkurbo_ffi.a

linux64:
	ADDON_LIBS = libs/kurbo-ffi/lib/linux64/libkurbo_ffi.so