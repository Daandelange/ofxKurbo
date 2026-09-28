
#include "ofxKurboGlue.h"

kurboPos to_kurboPos(const glm::vec2& _pos) {
    return kurboPos{ _pos.x, _pos.y };
}

glm::vec2 to_glmVec2(const kurboPos& _pos) {
    return glm::vec2(_pos.x, _pos.y);
}

std::vector<KurboPathEl> kurbo_elements_from_rect(const kurboRect& _rect) {
    std::vector<KurboPathEl> ret;
    ret.push_back({KurboPathElType::MoveTo, {_rect.x0, _rect.y0}, {0,0}, {0,0}});
    ret.push_back({KurboPathElType::LineTo, {_rect.x1, _rect.y0}, {0,0}, {0,0}});
    ret.push_back({KurboPathElType::LineTo, {_rect.x1, _rect.y1}, {0,0}, {0,0}});
    ret.push_back({KurboPathElType::LineTo, {_rect.x0, _rect.y1}, {0,0}, {0,0}});
    ret.push_back({KurboPathElType::ClosePath, {0,0}, {0,0}, {0,0}});
    return ret;
}

std::ostream & operator<< (std::ostream& out, kurboPos const& pos) {
    out << "[" << pos.x << "; " << pos.y << "]";
    return out;
}

std::string getBooleanOpString(const KurboBooleanOp& op){
    switch (op) {
        case KurboBooleanOp::Union:        return "Union";
        case KurboBooleanOp::Intersection: return "Intersection";
        case KurboBooleanOp::Difference:   return "Difference";
        case KurboBooleanOp::Xor:          return "Xor";
        default:                           return "Other";
    }
}
