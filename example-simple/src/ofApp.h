#pragma once

#include "ofMain.h"
#include <vector>
#include "ofxKurbo.h"

class ofApp : public ofBaseApp {
public:
    void setup();
    void update();
    void draw();
    void exit();

    void keyPressed(int key);
    void keyReleased(int key);
    void mouseMoved(int x, int y);
    void mouseDragged(int x, int y, int button);
    void mousePressed(int x, int y, int button);
    void mouseReleased(int x, int y, int button);
    void mouseEntered(int x, int y);
    void mouseExited(int x, int y);
    void windowResized(int w, int h);
    void dragEvent(ofDragInfo dragInfo);
    void gotMessage(ofMessage msg);

    // Kurbo path handling
    KurboBezPath* path;
    KurboBezPath* strokedPath;
    
    bool bShowHelp = true;
    bool bShowInfo = true;
    bool bAnimate = true;

    // Stroke options
    double strokeWidth = 10.0;
    kurboJoinType joinType = kurboJoinType::Round;
    kurboCapType startCap = kurboCapType::Round;
    kurboCapType endCap = kurboCapType::Round;
    double miterLimit = 4.0;
    bool bShowStroke = true;

    void rebuildPath();
};