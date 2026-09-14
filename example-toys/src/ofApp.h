#pragma once

#include "ofMain.h"
#include <vector>
#include "ofxKurbo.h"
#include "kurboShape.h"
#include "kurboToys.h"

class ofApp : public ofBaseApp {
public:
    void setup();
    void update();
    void draw();

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
    void generateNewShape(bool random=true);
    
    kurboShape shape;
    kurboShape fxShape;

    glm::vec2 clickedPos = {0,0};
    glm::vec2 bezierPreview = {0,0};

    unsigned int currentToy = 0;
    std::vector<kurboToy*> toys;

    bool bShowHelp = true;
    bool bShowInfo = true;
    bool bAnimate = true;
};