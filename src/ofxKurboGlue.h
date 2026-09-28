#pragma once

#include <vector>
#include "ofGraphicsBaseTypes.h"
#include "kurbo-ffi.h"

// Glue
kurboPos to_kurboPos(const glm::vec2& _pos);
glm::vec2 to_glmVec2(const kurboPos& _pos);
std::vector<KurboPathEl> kurbo_elements_from_rect(const kurboRect& _rect);

// Overload glue (ofToString, etc)
std::ostream & operator<< (std::ostream& out, kurboPos const& pos);

std::string getBooleanOpString(const KurboBooleanOp& op);
