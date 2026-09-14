#include "ofApp.h"

//--------------------------------------------------------------
void ofApp::setup() {
    ofSetFrameRate(60);
    ofSetWindowTitle("ofxKurbo Example");
    
    // Create initial path
    path = kurbo_path_create(nullptr);
    strokedPath = nullptr;
    
    // Add a simple shape (a curved path)
    kurbo_path_append_move_to(path, to_kurboPos(glm::vec2(100, 200)));
    kurbo_path_append_curve_to(path, to_kurboPos(glm::vec2(100, 400)), to_kurboPos(glm::vec2(400, 400)), to_kurboPos(glm::vec2(400, 200)));
    kurbo_path_append_curve_to(path, to_kurboPos(glm::vec2(400, 000)), to_kurboPos(glm::vec2(700, 000)), to_kurboPos(glm::vec2(700, 200)));
    // kurbo_path_append_close(path);
    
    rebuildPath();
}

//--------------------------------------------------------------
void ofApp::update() {
    if (bAnimate) {
        // Simple animation: modify stroke width
        strokeWidth = 10.0 + 15.0 * sin(ofGetElapsedTimef() * 2.0);
        rebuildPath();
    }
}

//--------------------------------------------------------------
void ofApp::draw() {
    ofBackground(30, 30, 30);
    
    // Draw original path
    ofNoFill();
    ofSetColor(100, 200, 255);
    ofSetLineWidth(2);
    
    KurboPathRaw rawData = kurbo_path_return_handle_data(path);
    ofBeginShape();
    for (size_t i = 0; i < rawData.len; i++) {
        const KurboPathEl& el = rawData.data[i];
        if (el.tag == KurboPathElType::MoveTo) {
            ofVertex(to_glmVec2(el.p0));
        } else if (el.tag == KurboPathElType::LineTo) {
            ofVertex(to_glmVec2(el.p0));
        } else if (el.tag == KurboPathElType::CurveTo) {
            ofBezierVertex(to_glmVec2(el.p0), to_glmVec2(el.p1), to_glmVec2(el.p2));
        } else if (el.tag == KurboPathElType::ClosePath) {
            ofEndShape(true);
        }
    }
    ofEndShape(false);
    
    // Draw stroked path if enabled
    if (bShowStroke && strokedPath != nullptr) {
        ofSetColor(255, 100, 100);
        ofBeginShape();
        KurboPathRaw strokeData = kurbo_path_return_handle_data(strokedPath);
        for (size_t i = 0; i < strokeData.len; i++) {
            const KurboPathEl& el = strokeData.data[i];
            if (el.tag == KurboPathElType::MoveTo) {
                ofVertex(to_glmVec2(el.p0));
            } else if (el.tag == KurboPathElType::LineTo) {
                ofVertex(to_glmVec2(el.p0));
            } else if (el.tag == KurboPathElType::CurveTo) {
                ofBezierVertex(to_glmVec2(el.p0), to_glmVec2(el.p1), to_glmVec2(el.p2));
            } else if (el.tag == KurboPathElType::ClosePath) {
                ofEndShape(true);
            }
        }
        ofEndShape(false);
    }
    
    // Draw UI
    if (bShowHelp) {
        ofSetColor(255);
        ofDrawBitmapString("ofxKurbo Example", 20, 30);
        ofDrawBitmapString("Press 'h' to toggle help", 20, 50);
        ofDrawBitmapString("Press 's' to toggle stroke visualization", 20, 70);
        ofDrawBitmapString("Press 'a' to toggle animation", 20, 90);
        ofDrawBitmapString("Stroke Width: " + ofToString(strokeWidth, 2), 20, 110);
    }
}

//--------------------------------------------------------------
void ofApp::rebuildPath() {
    if (strokedPath != nullptr) {
        kurbo_path_destroy(strokedPath);
    }
    strokedPath = kurbo_path_stroke(path, strokeWidth, joinType, miterLimit, startCap, endCap);
}

//--------------------------------------------------------------
void ofApp::exit() {
    if (path != nullptr) {
        kurbo_path_destroy(path);
        path = nullptr;
    }
    if (strokedPath != nullptr) {
        kurbo_path_destroy(strokedPath);
        strokedPath = nullptr;
    }
}

//--------------------------------------------------------------
void ofApp::keyPressed(int key) {
    if (key == 'h' || key == 'H') bShowHelp = !bShowHelp;
    else if (key == 's' || key == 'S') bShowStroke = !bShowStroke;
    else if (key == 'a' || key == 'A') bAnimate = !bAnimate;
}

void ofApp::keyReleased(int key) {}
void ofApp::mouseMoved(int x, int y) {}
void ofApp::mouseDragged(int x, int y, int button) {}
void ofApp::mousePressed(int x, int y, int button) {}
void ofApp::mouseReleased(int x, int y, int button) {}
void ofApp::mouseEntered(int x, int y) {}
void ofApp::mouseExited(int x, int y) {}
void ofApp::windowResized(int w, int h) {}
void ofApp::dragEvent(ofDragInfo dragInfo) {}
void ofApp::gotMessage(ofMessage msg) {}