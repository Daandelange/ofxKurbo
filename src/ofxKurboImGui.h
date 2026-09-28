
#pragma once

// Auto-enable according to ofxImGui global flag
#ifndef OFXKURBO_DEFINE_IMGUI_HELPERS
#   ifdef ofxAddons_ENABLE_IMGUI
#       define OFXKURBO_DEFINE_IMGUI_HELPERS
#   endif
#endif

// The imgui widgets are opt-in
#ifdef OFXKURBO_DEFINE_IMGUI_HELPERS

namespace ImGuiEx {
    void ofxKurboJointCombo(const char* _name, kurboJoinType& _joinType, double* _miter = nullptr);
    void ofxKurboStrokeOptions(const char* _name, double& _width, kurboJoinType& _joinType, kurboCapType& _startCap, kurboCapType& _endCap, double* _miter = nullptr);
}
#endif
