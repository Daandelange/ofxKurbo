#pragma once
#include "kurbo-ffi.h"
#include <vector>
#include "ofGraphicsBaseTypes.h"

// Glue
kurboPos to_kurboPos(const glm::vec2& _pos);
glm::vec2 to_glmVec2(const kurboPos& _pos);
std::vector<KurboPathEl> kurbo_elements_from_rect(const kurboRect& _rect);

// Overload glue (ofToString, etc)
std::ostream & operator<< (std::ostream& out, kurboPos const& pos);

// Auto-enable according to ofxImGui global flag
#ifndef OFXKURBO_DEFINE_IMGUI_HELPERS
#   ifdef ofxAddons_ENABLE_IMGUI
#       define OFXKURBO_DEFINE_IMGUI_HELPERS
#   endif
#endif

#ifdef OFXKURBO_DEFINE_IMGUI_HELPERS
namespace ImGuiEx {
    void ofxKurboJointCombo(const char* _name, kurboJoinType& _joinType, double* _miter = nullptr);
    void ofxKurboStrokeOptions(const char* _name, double& _width, kurboJoinType& _joinType, kurboCapType& _startCap, kurboCapType& _endCap, double* _miter = nullptr);
}
#endif