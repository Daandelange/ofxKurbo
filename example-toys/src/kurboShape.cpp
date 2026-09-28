#include "ofMain.h"
#include "kurboShape.h"

bool kurboShape::bShowNumbers = true;

void kurboShape::draw(bool filled, ofColor lineColor) const {
    // Set style
    ofSetColor(lineColor);//, 255);
    if(filled){
        ofFill();
    }
    else {
        ofNoFill();
        ofSetLineWidth(2);
    }
    // Draw the path
    ofBeginShape();
    for (const KurboPathEl& el : this->elements) {
        if (el.tag == KurboPathElType::MoveTo){ ofEndShape(false); ofVertex(to_glmVec2(el.p0)); }
        else if (el.tag == KurboPathElType::LineTo) ofVertex(to_glmVec2(el.p0));
        else if (el.tag == KurboPathElType::QuadTo) ofBezierVertex(to_glmVec2(el.p0), to_glmVec2(el.p0), to_glmVec2(el.p1));
        else if (el.tag == KurboPathElType::CurveTo) ofBezierVertex(to_glmVec2(el.p0), to_glmVec2(el.p1), to_glmVec2(el.p2));
        else if (el.tag == KurboPathElType::ClosePath) { ofEndShape(true); ofBeginShape(); }
    }
    ofEndShape(false);
    
    if(!filled){
        ofSetLineWidth(1);
    }
}

void kurboShape::drawBezierHandles(ofColor handlesColor, ofColor anchorColor) const {
    // Visualise vector data
    ofSetLineWidth(1);
    ofSetColor(handlesColor);
    // int i = 0;
    int vertexIndex = 0;
    glm::vec2 currentPt = {0, 0};
    glm::vec2 subpathStart = {0, 0};

    for (const KurboPathEl& el : this->elements) {
        if (el.tag == KurboPathElType::MoveTo || el.tag == KurboPathElType::LineTo) {
            currentPt = to_glmVec2(el.p0);
            ofFill();
            ofSetColor(anchorColor);
            ofDrawCircle(currentPt.x, currentPt.y, 3);

            if(el.tag == KurboPathElType::MoveTo){
                subpathStart = currentPt;
            }
        }
        else if (el.tag == KurboPathElType::QuadTo || el.tag == KurboPathElType::CurveTo) {
            glm::vec2 cp1 = to_glmVec2(el.p0); // Out-handle of start anchor
            glm::vec2 endPt = to_glmVec2(el.p2);

            // Draw control point
            ofFill();
            ofSetColor(handlesColor);
            ofDrawCircle(cp1.x, cp1.y, 2);

            if(el.tag == KurboPathElType::CurveTo){
                glm::vec2 cp2 = to_glmVec2(el.p1); // In-handle of end anchor
                ofDrawCircle(cp2.x, cp2.y, 2);

                // Draw handle lines (connecting out-handle to start anchor, and in-handle to end anchor)
                ofNoFill();
                ofSetColor(handlesColor);
                ofDrawLine(currentPt.x, currentPt.y, cp1.x, cp1.y);
                ofDrawLine(endPt.x, endPt.y, cp2.x, cp2.y);
            }
            
            else {
                // Draw handle lines (connecting anchors to the control point)
                ofNoFill();
                ofSetColor(handlesColor);
                ofDrawLine(currentPt.x, currentPt.y, cp1.x, cp1.y);
                ofDrawLine(cp1.x, cp1.y, endPt.x, endPt.y);
            }
            
            // Draw end anchor
            ofFill();
            ofSetColor(anchorColor);
            ofDrawCircle(endPt.x, endPt.y, 3);
            
            currentPt = endPt;
        } 
        else if (el.tag == KurboPathElType::ClosePath) {
            // For a closed path, the end anchor is the start of the subpath.
            // We don't draw an extra anchor here to avoid duplicating the MoveTo anchor.
            currentPt = subpathStart;
            //vertexIndex = 0;
        }

        // Show numbers ?
        if (kurboShape::bShowNumbers && el.tag != KurboPathElType::ClosePath) {
            glm::vec2 offset = {5, 5};
            ofDrawBitmapStringHighlight(ofToString(vertexIndex), currentPt.x + offset.x, currentPt.y + offset.y, ofColor(anchorColor,80), ofColor(255,255,255,200));
            vertexIndex++;
        }   
    }
    if constexpr (false) for (const KurboPathEl& el : this->elements) {
        if (el.tag == KurboPathElType::MoveTo) {
            currentPt = to_glmVec2(el.p0);
            subpathStart = currentPt;
            
            // Draw anchor
            ofFill(); ofSetColor(anchorColor); ofDrawCircle(currentPt.x, currentPt.y, 3);
            
            // Draw number
            if (kurboShape::bShowNumbers) {
                glm::vec2 offset = {5, 5};
                if (vertexIndex == 0) offset *= -1; // Flip offset for the very first vertex
                ofDrawBitmapStringHighlight(ofToString(vertexIndex), currentPt.x + offset.x, currentPt.y + offset.y, ofColor(0,0,0,80), ofColor(255,255,255,200));
            }
            // vertexIndex++;
            vertexIndex = 0;
        } 
        else if (el.tag == KurboPathElType::LineTo) {
            currentPt = to_glmVec2(el.p0);
            
            // Draw anchor
            ofFill(); ofSetColor(anchorColor); ofDrawCircle(currentPt.x, currentPt.y, 3);
            
            // Draw number
            if (kurboShape::bShowNumbers) {
                glm::vec2 offset = {5, 5};
                ofDrawBitmapStringHighlight(ofToString(vertexIndex), currentPt.x + offset.x, currentPt.y + offset.y, ofColor(0,0,0,80), ofColor(255,255,255,200));
            }
            vertexIndex++;
        } 
        else if (el.tag == KurboPathElType::QuadTo) {
            glm::vec2 cp1 = to_glmVec2(el.p0); // Control point
            glm::vec2 endPt = to_glmVec2(el.p1); // Anchor point
            
            // Draw control point
            ofFill(); ofSetColor(handlesColor, 255); ofDrawCircle(cp1.x, cp1.y, 2);
            
            // Draw handle lines
            ofNoFill(); ofSetColor(handlesColor, 255);
            ofDrawLine(currentPt.x, currentPt.y, cp1.x, cp1.y);
            ofDrawLine(cp1.x, cp1.y, endPt.x, endPt.y);
            
            currentPt = endPt;
            
            // Draw anchor
            ofFill(); ofSetColor(anchorColor); ofDrawCircle(currentPt.x, currentPt.y, 3);
            
            // Draw number
            if (kurboShape::bShowNumbers) {
                glm::vec2 offset = {5, 5};
                ofDrawBitmapStringHighlight(ofToString(vertexIndex), currentPt.x + offset.x, currentPt.y + offset.y, ofColor(0,0,0,80), ofColor(255,255,255,200));
            }
            vertexIndex++;
        } 
        else if (el.tag == KurboPathElType::CurveTo) {
            glm::vec2 cp1 = to_glmVec2(el.p0); // Out-handle of start anchor
            glm::vec2 cp2 = to_glmVec2(el.p1); // In-handle of end anchor
            glm::vec2 endPt = to_glmVec2(el.p2); // Anchor point
            
            // Draw control points
            ofFill(); ofSetColor(handlesColor, 255);
            ofDrawCircle(cp1.x, cp1.y, 2);
            ofDrawCircle(cp2.x, cp2.y, 2);
            
            // Draw handle lines
            ofNoFill(); ofSetColor(handlesColor, 255);
            ofDrawLine(currentPt.x, currentPt.y, cp1.x, cp1.y);
            ofDrawLine(endPt.x, endPt.y, cp2.x, cp2.y);
            
            currentPt = endPt;
            
            // Draw anchor
            ofFill(); ofSetColor(anchorColor); ofDrawCircle(currentPt.x, currentPt.y, 3);
            
            // Draw number
            if (kurboShape::bShowNumbers) {
                glm::vec2 offset = {5, 5};
                ofDrawBitmapStringHighlight(ofToString(vertexIndex), currentPt.x + offset.x, currentPt.y + offset.y, ofColor(0,0,0,80), ofColor(255,255,255,200));
            }
            vertexIndex++;
        }
        else if (el.tag == KurboPathElType::ClosePath) {
            // ClosePath doesn't introduce a new vertex. 
            // We just reset currentPt to the start of the subpath for visual continuity.
            currentPt = subpathStart;
        }
    }
}
