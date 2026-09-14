#pragma once

#include "ofColor.h"
#include "ofxKurbo.h"
#include <vector>

struct kurboShape {
    std::vector<KurboPathEl> elements = {};
    bool bChanged = false;
    static bool bShowNumbers;

    void draw(bool connectLast=true, ofColor lineColor=ofColor::black, ofColor bezierColor=ofColor::blue);
};