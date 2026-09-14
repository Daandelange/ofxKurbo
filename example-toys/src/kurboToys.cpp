#include "ofMain.h"
#include "ofGraphics.h"
#include "ofAppRunner.h"
#include "ofUtils.h"
#include "ofMath.h"
#include "ofVectorMath.h"
#include "ofxKurbo.h"
#include "kurboToys.h"

#define HUD_BG_ALPHA 200

// Math helpers
#include <cmath>
inline double getModuloTime(double _interval = 1.){
    return std::fmod(ofGetElapsedTimef()/_interval, 1.);
}
inline float getSineTime(float _interval = 1.f){
    return std::sin(ofGetElapsedTimef()*TWO_PI/_interval);
}
// Gui helpers
std::string getBezrsJoinString(kurboJoinType join){
    std::string joinString("");
    joinString += (join==kurboJoinType::Bevel?"Bevel":(join==kurboJoinType::Miter?"Miter":(join==kurboJoinType::Round?"Round":"Other")));
    if(join == kurboJoinType::Round) joinString += " (unstable)";
    return joinString;
}

inline glm::vec2 getTextPosStart(){
    return {50, ofGetHeight() - 50};
}
constexpr int lineHeight = 30;

//--------------------------------------------------------------
void kurboToy::drawParams(const kurboShape& _sh){
    // Nothing drawn by default...
}

const char* kurboToy::name_cstr(){
    return name.c_str();
}

//--------------------------------------------------------------
// Copies shape to raw handle then sends it to bezRS (Rust)
// Returned handle needs to be freed later !
inline KurboBezPath* sendShapeToKurbo(const kurboShape& _inShape) {
    KurboPathRaw rawInput = { _inShape.elements.data(), _inShape.elements.size() };
    return kurbo_path_create(&rawInput);
}

// Retrieves bezrs shape data to oF (c++)
// Destroys handle too
inline void populateShapeFromKurbo(KurboBezPath* kPath, kurboShape& _outShape, bool destroyPath=true) {
    KurboPathRaw rawData = kurbo_path_return_handle_data(kPath);
    _outShape.elements.clear();
    for (size_t i = 0; i < rawData.len; i++) {
        _outShape.elements.push_back(rawData.data[i]);
    }
    if (destroyPath) kurbo_path_destroy(kPath);
}


//--------------------------------------------------------------
void offsetToy::applyFX(const kurboShape& _inShape, kurboShape& _outShape) {
    updateParams();

    // Create internal handle
    KurboBezPath* path = sendShapeToKurbo(_inShape);

    // Apply transform
    KurboBezPath* offsetPath = kurbo_path_offset(path, offset, 0.1);

    // Retrieve result
    populateShapeFromKurbo(offsetPath, _outShape, true);
    kurbo_path_destroy(path);
}

void offsetToy::updateParams() {
    static const float cycle = 10.f;
    offset = getSineTime(cycle)*lineHeight;

    // Move joint type every cyle
    unsigned int now = ofGetElapsedTimef()/cycle;
    if( now != lastTime){
        lastTime = now;
        switch (join) {
            case kurboJoinType::Bevel :
                join = kurboJoinType::Miter;
                break;
            case kurboJoinType::Miter :
                join = kurboJoinType::Round;
                break;
            case kurboJoinType::Round :
            default:
                join = kurboJoinType::Bevel;
                break;
        }
    }
}

void offsetToy::drawParams(const kurboShape& _sh) {
    glm::vec2 textPos = getTextPosStart();
    ofDrawBitmapStringHighlight("Offsets the shape by an amount in pixels.", textPos.x, textPos.y);
    textPos.y -= lineHeight;
    ofDrawBitmapStringHighlight("Beware the winding order : CCW reverses the direction and creates some artifacts.", textPos.x, textPos.y);
    textPos.y -= lineHeight;
    ofDrawBitmapStringHighlight(ofToString("Offset = ")+ofToString(offset), textPos.x, textPos.y, ofColor(ofColor::red, HUD_BG_ALPHA));
    textPos.y -= lineHeight;
    ofDrawBitmapStringHighlight(ofToString("Join   = ")+getBezrsJoinString(join), textPos.x, textPos.y);
}

//--------------------------------------------------------------
void outlineToy::applyFX(const kurboShape& _inShape, kurboShape& _outShape) {
    updateParams();

    // Create internal handle
    KurboBezPath* path = sendShapeToKurbo(_inShape);

    // Transform the shape
    KurboBezPath* stroked = kurbo_path_stroke(path, offset, join, 0, kurboCapType::Round, kurboCapType::Round);
    if (stroked != nullptr) {
        // Retrieve and destroy internal handle
        populateShapeFromKurbo(stroked, _outShape, true);
    }

    kurbo_path_destroy(path);
}

void outlineToy::drawParams(const kurboShape& _sh) {
    glm::vec2 textPos = getTextPosStart();
    ofDrawBitmapStringHighlight("Creates an outline of a shape at a given distance.", textPos.x, textPos.y);
    textPos.y -= lineHeight;
    ofDrawBitmapStringHighlight("Beware the winding order : CCW reverses the direction and creates some artifacts.", textPos.x, textPos.y);
    textPos.y -= lineHeight;
    ofDrawBitmapStringHighlight(ofToString("Distance = ")+ofToString(offset), textPos.x, textPos.y, ofColor(ofColor::red, HUD_BG_ALPHA));
    textPos.y -= lineHeight;
    ofDrawBitmapStringHighlight(ofToString("Join     = ")+getBezrsJoinString(join), textPos.x, textPos.y);
}

//--------------------------------------------------------------
void rotationToy::applyFX(const kurboShape& _inShape, kurboShape& _outShape) {
    // Update params
    center.x = ofGetWidth()*.5;
    center.y = ofGetHeight()*.5;
    rotation = getModuloTime(10.f)*TWO_PI;

    KurboBezPath* path = sendShapeToKurbo(_inShape);
    kurbo_path_rotate(path, rotation, center);
    populateShapeFromKurbo(path, _outShape, true);
}

void rotationToy::drawParams(const kurboShape& _sh) {

    glm::vec2 textPos = {50, ofGetHeight() - 50};
    ofDrawBitmapStringHighlight("Rotates the shape (in radians) around a center point.", textPos.x, textPos.y);
    textPos.y -= lineHeight;
    ofDrawBitmapStringHighlight(ofToString("Rotation = ")+ofToString(rotation/TWO_PI), textPos.x, textPos.y, ofColor(ofColor::red, HUD_BG_ALPHA));
    textPos.y -= lineHeight;
    ofDrawBitmapStringHighlight(ofToString("Center   = ")+ofToString(center), textPos.x, textPos.y, ofColor(ofColor::green, HUD_BG_ALPHA));

    // Visualise center
    ofNoFill();
    ofSetColor(ofColor::green);
    const static int crossSize = 6;
    ofSetLineWidth(2);
    ofDrawLine(center.x - crossSize, center.y, center.x+crossSize, center.y);
    ofDrawLine(center.x, center.y-crossSize, center.x, center.y+crossSize);
    ofSetLineWidth(1);
}

//--------------------------------------------------------------
void reverseWindingToy::applyFX(const kurboShape& _inShape, kurboShape& _outShape) {
    reversed = getSineTime(10.f) >= 0.f;

    if(ofGetKeyPressed('r')){
        reversed = !reversed;
    }

    if(reversed){
        KurboBezPath* path = sendShapeToKurbo(_inShape);
        kurbo_path_reverse(path);
        populateShapeFromKurbo(path, _outShape, true);
    }
}

void reverseWindingToy::drawParams(const kurboShape& _sh) {
    glm::vec2 textPos = {50, ofGetHeight() - 50};
    ofDrawBitmapStringHighlight("Reverses the shape winding.", textPos.x, textPos.y);
    textPos.y -= 30;
    ofDrawBitmapStringHighlight(ofToString("Reversed = ")+(reversed?"yes":"no"), textPos.x, textPos.y, ofColor(ofColor::red, HUD_BG_ALPHA));
    textPos.y -= 30;
}

//--------------------------------------------------------------
void boundingBoxToy::applyFX(const kurboShape& _inShape, kurboShape& _outShape) {
    // Create internal handle
    KurboBezPath* path = sendShapeToKurbo(_inShape);

    // Retrieve boundingbox
    bb = kurbo_path_boundingbox(path);

    // Destroy handle
    kurbo_path_destroy(path);
}

void boundingBoxToy::drawParams(const kurboShape& _sh) {
    // Gui
    glm::vec2 textPos = {50, ofGetHeight() - 50};
    ofDrawBitmapStringHighlight("Calculates the bounding box that contains the shape (red).", textPos.x, textPos.y);
    textPos.y -= 30;
    ofDrawBitmapStringHighlight(std::string("Position = ") + ofToString(glm::vec2(bb.x0, bb.y0)) , textPos.x, textPos.y);
    textPos.y -= 30;
    ofDrawBitmapStringHighlight(std::string("Size     = ") + ofToString(glm::vec2(bb.x1-bb.x0, bb.y1-bb.y0)) , textPos.x, textPos.y);
    textPos.y -= 30;

    // Visualise
    ofNoFill();
    ofSetColor(ofColor::red);
    ofDrawRectangle(bb.x0, bb.y0, bb.x1 - bb.x0, bb.y1 - bb.y0);
}

//--------------------------------------------------------------
void hitTestToy::applyFX(const kurboShape& _inShape, kurboShape& _outShape) {
    // Create internal handle
    KurboBezPath* path = sendShapeToKurbo(_inShape);
    
    // Update params
    mousePos = glm::vec2(ofGetMouseX(), ofGetMouseY());
    // Animate simPos
    simPos += simVec;
    static const float mouseHitTolerance = 10; // px
    // Change the velocity when mouse close-to ?
    if(glm::distance(simPos, mousePos) <= mouseHitTolerance){
        float randAngle = ofRandom(0.,TWO_PI);
        simVec = glm::vec2(sin(randAngle)*simSpeed,cos(randAngle)*simSpeed);
    }
    else {
        static int outOfBounds = 0;
        const bool outOfX = simPos.x < 0 || simPos.x > ofGetWidth();
        const bool outOfY = simPos.y < 0 || simPos.y > ofGetHeight();

        // Bounce ?
        if(outOfX) simVec.x *=-1;
        if(outOfY) simVec.y *=-1;

        // Respawn ?
        if(outOfX || outOfY){
            outOfBounds++;
        }
        else {
            outOfBounds = 0;
        }
        if(outOfBounds>30){
            simPos = {ofRandom(0,ofGetWidth()), ofRandom(0,ofGetHeight())};
            outOfBounds = 0;
        }
    }

    // Query kurbo
    // Do hit tests
    mousePosHit = kurbo_path_contains_point(path, to_kurboPos(mousePos));
    simPosHit = kurbo_path_contains_point(path, to_kurboPos(simPos));
    // Project points
    mouseProjection = to_glmVec2(kurbo_path_project(path, to_kurboPos(mousePos), 0.1));
    simProjection = to_glmVec2(kurbo_path_project(path, to_kurboPos(simPos), 0.1));

    // Cleanup
    kurbo_path_destroy(path);
}

void hitTestToy::drawParams(const kurboShape& _sh) {
    glm::vec2 textPos = {50, ofGetHeight() - 50};
    ofDrawBitmapStringHighlight("Checks if a point is contained in the shape.", textPos.x, textPos.y);
    textPos.y -= 30;
    ofDrawBitmapStringHighlight("Also finds the closest point on the curve, aka. projection.", textPos.x, textPos.y);
    textPos.y -= 30;
    ofDrawBitmapStringHighlight(mousePosHit ? "Mouse Inside" : "Mouse Outside", textPos.x, textPos.y, ofColor(ofColor::red, HUD_BG_ALPHA));
    textPos.y -= 30;
    ofDrawBitmapStringHighlight(simPosHit ? "Sim Inside" : "Sim Outside", textPos.x, textPos.y, ofColor(ofColor::green, HUD_BG_ALPHA));

    // Draw positions
    ofFill();
    ofSetLineWidth(2);
    ofSetColor(!simPosHit?ofColor::darkGreen:ofColor(ofColor::green, 100));
    ofDrawCircle(simPos, 5.f);
    ofDrawLine(simPos, simProjection);
    ofSetColor(!mousePosHit?ofColor::darkRed:ofColor(ofColor::red,100));
    ofDrawCircle(mousePos, 5.f);
    ofDrawLine(mousePos, mouseProjection);
    ofSetLineWidth(1);
    ofNoFill();
}

//--------------------------------------------------------------
void inflectionsToy::applyFX(const kurboShape& _inShape, kurboShape& _outShape) {
    KurboBezPath* path = sendShapeToKurbo(_inShape);
    
    kurboFloatsRaw infs = kurbo_path_inflections(path);
    inflections.clear();
    for (size_t i = 0; i < infs.len; i++) {
        kurboEvalResult res = kurbo_path_evaluate(path, infs.data[i]);
        inflections.push_back(to_glmVec2(res.pos));
    }

    kurboFloatsRaw exts = kurbo_path_extrema(path);
    local_extremas.clear();
    for (size_t i = 0; i < exts.len; i++) {
        kurboEvalResult res = kurbo_path_evaluate(path, exts.data[i]);
        local_extremas.push_back(to_glmVec2(res.pos));
    }

    kurbo_path_destroy(path);
}

void inflectionsToy::drawParams(const kurboShape& _sh) {
    // Gui
    glm::vec2 textPos = {50, ofGetHeight() - 50};
    ofDrawBitmapStringHighlight("Calculates inflection points on the shape.", textPos.x, textPos.y);
    textPos.y -= 30;
    ofDrawBitmapStringHighlight("Inflection points : "+ofToString(inflections.size()), textPos.x, textPos.y, ofColor(ofColor::green, HUD_BG_ALPHA));
    textPos.y -= 30;
    ofDrawBitmapStringHighlight("Local extremas    : "+ofToString(local_extremas.size()), textPos.x, textPos.y, ofColor(ofColor::violet, HUD_BG_ALPHA));
    textPos.y -= 30;

    // Visualise inflections
    ofFill();
    ofSetColor(ofColor::green);
    for (const auto& pt : inflections) {
        ofDrawCircle(pt, 4);
    }

    // Visualise local extremas
    ofSetColor(ofColor::violet);
    for(const auto& ip : local_extremas){
        ofDrawCircle(ip.x, ip.y, 4);
    }
}

//--------------------------------------------------------------
void evaluateToy::applyFX(const kurboShape& _inShape, kurboShape& _outShape) {
    // Update params
    // tval += 0.005;
    // if (tval > 1.0) tval = 0.0;
    static const float cycle = 10.f;
    tval = getModuloTime(cycle);
    static float prevTval = 0;
    if(tval<prevTval){ // looped ?
        bEuclidean = !bEuclidean;
    }
    prevTval = tval;

    // Custom control
    static bool bWasPressed = false;
    if(ofGetKeyPressed('e')){
        // ignores repeats
        if(!bWasPressed) bEuclidean = !bEuclidean;
        bWasPressed= true;
    }
    else {
        bWasPressed = false;
    }
    // Tmp change tvalue with mouse
    if(ofGetMousePressed()){
        tval = ((float)ofGetMouseX())/ofGetWidth();
    }

    KurboBezPath* path = sendShapeToKurbo(_inShape);
    if (bEuclidean) {
        evalRes = kurbo_path_evaluate_euclidean(path, tval, 0.1);
    } else {
        evalRes = kurbo_path_evaluate(path, tval);
    }
    kurbo_path_destroy(path);
}

void evaluateToy::drawParams(const kurboShape& _sh) {
    // Gui
    glm::vec2 textPos = {50, ofGetHeight() - 50};
    ofDrawBitmapStringHighlight("Evaluates a t-value position.", textPos.x, textPos.y);
    textPos.y -= 30;
    // ofDrawBitmapStringHighlight(std::string("Green = Tangent, Orange = Normal, Pink=Curvature"), textPos.x, textPos.y);
    // textPos.y -= 30;
    ofDrawBitmapStringHighlight(std::string("Tangent   = ") + ofToString(evalRes.tangent) + " / length="+ofToString(glm::length2(glm::vec2(evalRes.tangent.x, evalRes.tangent.y)))+"px", textPos.x, textPos.y, ofColor(ofColor::red, HUD_BG_ALPHA));
    textPos.y -= 30;
    ofDrawBitmapStringHighlight(std::string("Curvature = ") + ofToString(evalRes.curvature), textPos.x, textPos.y, ofColor(ofColor::violet, HUD_BG_ALPHA));
    textPos.y -= 30;
    ofDrawBitmapStringHighlight(std::string("Normal    = ") + ofToString(evalRes.normal) + " / length="+ofToString(glm::length2(glm::vec2(evalRes.normal.x, evalRes.normal.y)))+"px", textPos.x, textPos.y, ofColor(ofColor::orange, HUD_BG_ALPHA));
    textPos.y -= 30;
    ofDrawBitmapStringHighlight(std::string("Position  = ") + ofToString(evalRes.pos), textPos.x, textPos.y, ofColor(ofColor::green, HUD_BG_ALPHA));
    textPos.y -= 30;
    ofDrawBitmapStringHighlight(std::string("TValue    = ") + ofToString(tval), textPos.x, textPos.y);
    textPos.y -= 30;
    ofDrawBitmapStringHighlight(std::string("Mode (e)  = ") + (bEuclidean ? "Euclidean (arc-length)" : "Linear (segment-based)"), textPos.x, textPos.y);
    textPos.y -= 30;

    // Curvature
    ofSetLineWidth(3);
    // Calc  normalized values
    const glm::vec2 tNormal = glm::normalize(glm::vec2(evalRes.normal.x, evalRes.normal.y));
    // const glm::vec2 tTangent = glm::normalize(glm::vec2(evalRes.tangent.x, evalRes.tangent.y));

    // Visualise data
    ofNoFill();
    ofSetColor(ofColor::violet);
    if( std::abs(evalRes.curvature) > 0.001 ){ // close to 0 means infinite !
        double radius = 1./evalRes.curvature;
        ofDrawCircle(evalRes.pos.x+tNormal.x*radius, evalRes.pos.y+tNormal.y*radius, std::abs((float)radius));
        ofDrawLine(evalRes.pos.x, evalRes.pos.y, evalRes.pos.x+tNormal.x*radius, evalRes.pos.y+tNormal.y*radius);
    }
    
    // Tangent
    static const double mult = 0.2;
    ofSetColor(ofColor::red);
    ofDrawLine(evalRes.pos.x, evalRes.pos.y, evalRes.pos.x+evalRes.tangent.x*mult, evalRes.pos.y+evalRes.tangent.y*mult);

    // Normal
    static const double normalSize = 20.;
    ofSetColor(ofColor::orange);
    ofDrawLine(
        evalRes.pos.x+tNormal.x*normalSize*-.5, evalRes.pos.y+tNormal.y*normalSize*-.5,
        evalRes.pos.x+tNormal.x*normalSize*+.5, evalRes.pos.y+tNormal.y*normalSize*+.5
    );
    ofSetLineWidth(1); // restore

    // Draw evaluated position
    ofSetColor(ofColor::green);
    ofFill();
    ofDrawCircle(evalRes.pos.x, evalRes.pos.y, 3);
}

//--------------------------------------------------------------
void selfIntersectToy::applyFX(const kurboShape& _inShape, kurboShape& _outShape) {
    // Update Params
    static const float cycle = 10.f;
    offset = getSineTime(cycle)*80.f; // to animate

    // Get handles
    KurboBezPath* path = sendShapeToKurbo(_inShape);
    KurboBezPath* offsetPath = kurbo_path_offset(path, offset, 0.1);
    
    // Compute
    kurboFloatsRaw ints = kurbo_path_self_intersections(offsetPath, 3.5, 1.);
    selfIntersects.clear();
    floatsVec.clear();
    for (size_t i = 0; i < ints.len; i++) {
        floatsVec.push_back(ints.data[i]);
        kurboEvalResult res = kurbo_path_evaluate(offsetPath, ints.data[i]);
        // kurboEvalResult res = kurbo_path_evaluate_euclidean(offsetPath, ints.data[i], 0.1);
        selfIntersects.push_back(to_glmVec2(res.pos));
    }
    
    // Grab data & cleanup handles
    populateShapeFromKurbo(offsetPath, _outShape);
    kurbo_path_destroy(path);
}

void selfIntersectToy::drawParams(const kurboShape& _sh) {
    // Gui
    glm::vec2 textPos = {50, ofGetHeight() - 50};
    ofDrawBitmapStringHighlight("Finds self intersections in offset (to detect errors).", textPos.x, textPos.y);
    textPos.y -= 30;
    ofDrawBitmapStringHighlight( ofToString("Offset            : ")+ofToString(offset), textPos.x, textPos.y, ofColor(ofColor::red, HUD_BG_ALPHA));
    textPos.y -= 30;
    ofDrawBitmapStringHighlight(std::string("Self intersections: ")+ofToString(selfIntersects.size()), textPos.x, textPos.y, ofColor(ofColor::green, HUD_BG_ALPHA));
    textPos.y -= 30;
    
    // Visualise
    ofFill();
    ofSetColor(ofColor::green);
    for (auto& pt : selfIntersects) {
        ofDrawCircle(pt, 5);
    }
    // tvalue bar (tmp?)
    ofNoFill();
    ofSetColor(ofColor::cyan);
    const glm::vec2 size = {ofGetHeight()*.5, 10};
    ofDrawRectangle(0,0,size.x,size.y);
    if(floatsVec.size()>0){
        for(double& p : floatsVec){
            ofDrawCircle(size.x*p, size.y*.5, size.y*.5);
        }
        ofDrawRectangle(0,size.y,size.x*0.01*floatsVec.size(),size.y*.2);
    }
}