
#include "ofxKurboImGui.h"

#ifdef OFXKURBO_DEFINE_IMGUI_HELPERS

#include "imgui.h"
#include <map>
#include <string>

void ofxKurboImGuiHelpMarker(const char* desc) {
    ImGui::SameLine();
    ImGui::TextDisabled("[?]");
    if (ImGui::IsItemHovered()) {
        ImGui::BeginTooltip();
        ImGui::PushTextWrapPos(ImGui::GetFontSize() * 35.0f);
        ImGui::TextUnformatted(desc);
        ImGui::PopTextWrapPos();
        ImGui::EndTooltip();
    }
}

void ImGuiEx::ofxKurboJointCombo(const char* _name, kurboJoinType& _joinType, double* _miter) {
    const static std::map<kurboJoinType, const std::string> joinMap = {
        {kurboJoinType::Bevel, "Bevel"},
        {kurboJoinType::Miter, "Miter"},
        {kurboJoinType::Round, "Round"}
    };

    if (ImGui::BeginCombo(_name, joinMap.at((const kurboJoinType)_joinType).c_str(), ImGuiComboFlags_None)) {
        for(auto j : joinMap) {
            const bool is_selected = (_joinType == j.first);
            if (ImGui::Selectable(j.second.c_str(), is_selected))
                _joinType = j.first;
            if (is_selected) ImGui::SetItemDefaultFocus();
        }
        ImGui::EndCombo();

        switch(_joinType) {
            case kurboJoinType::Bevel:
                ofxKurboImGuiHelpMarker("Slices off joins.");
                break;
            case kurboJoinType::Miter:
                ofxKurboImGuiHelpMarker("Extends joins outwards using a given threshold.");
                break;
            case kurboJoinType::Round:
                ofxKurboImGuiHelpMarker("Rounds the joins.");
                break;
        }
    }
    if(_miter != nullptr && _joinType == kurboJoinType::Miter) {
        ImGui::Indent();
        const static double miter_min = 0;
        ImGui::DragScalar("Miter", ImGuiDataType_Double, _miter, 0.1f, &miter_min);
        ofxKurboImGuiHelpMarker("Sets the maximum distance of the created miter.");
        ImGui::Unindent();
    }
}

void ImGuiEx::ofxKurboStrokeOptions(const char* _name, double& _width, kurboJoinType& _joinType, kurboCapType& _startCap, kurboCapType& _endCap, double* _miter) {
    ImGui::PushID(_name);
    ImGui::DragScalar("Width", ImGuiDataType_Double, &_width, 0.1);
    ofxKurboImGuiHelpMarker("Stroke width.");
    
    const static std::map<kurboCapType, const std::string> capMap = {
        {kurboCapType::Butt, "Butt"},
        {kurboCapType::Round, "Round"},
        {kurboCapType::Square, "Square"}
    };

    if (ImGui::BeginCombo("Start Cap", capMap.at((const kurboCapType)_startCap).c_str(), ImGuiComboFlags_None)) {
        for(auto c : capMap) {
            const bool is_selected = (_startCap == c.first);
            if (ImGui::Selectable(c.second.c_str(), is_selected)) _startCap = c.first;
            if (is_selected) ImGui::SetItemDefaultFocus();
        }
        ImGui::EndCombo();
    }

    if (ImGui::BeginCombo("End Cap", capMap.at((const kurboCapType)_endCap).c_str(), ImGuiComboFlags_None)) {
        for(auto c : capMap) {
            const bool is_selected = (_endCap == c.first);
            if (ImGui::Selectable(c.second.c_str(), is_selected)) _endCap = c.first;
            if (is_selected) ImGui::SetItemDefaultFocus();
        }
        ImGui::EndCombo();
    }

    ImGuiEx::ofxKurboJointCombo("Join Type", _joinType, _miter);
    ImGui::PopID();
}
#endif
