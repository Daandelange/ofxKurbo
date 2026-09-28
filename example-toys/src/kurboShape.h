#pragma once

#include "ofColor.h"
#include "ofxKurbo.h"
#include <vector>

struct kurboShape {
    std::vector<KurboPathEl> elements = {};
    bool bChanged = false;
    static bool bShowNumbers;

    void draw(bool filled=false, ofColor lineColor=ofColor::black) const;
    void drawBezierHandles(ofColor handlesColor=ofColor::blue, ofColor anchorColor=ofColor::black) const;
};
