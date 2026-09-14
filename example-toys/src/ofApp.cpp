#include "ofApp.h"

void ofApp::setup() {
    ofSetFrameRate(60);
    ofSetWindowTitle("ofxKurbo Example");

    // Better GFX
    ofEnableSmoothing();
    ofEnableAntiAliasing();
    ofSetCurveResolution(50);
    ofSetCircleResolution(50);

    toys.push_back(new offsetToy());
    toys.push_back(new outlineToy());
    toys.push_back(new rotationToy());
    toys.push_back(new reverseWindingToy());
    toys.push_back(new boundingBoxToy());
    toys.push_back(new hitTestToy());
    toys.push_back(new inflectionsToy());
    toys.push_back(new evaluateToy());
    toys.push_back(new selfIntersectToy());

    generateNewShape();
}

void ofApp::update() {
    // Update
    bool invertAction = ofGetKeyPressed(OF_KEY_RETURN);
    if(
        (bAnimate != invertAction ) || // Force change ?
        shape.bChanged // Changed
    ){
        fxShape = {};// clear data

        if(shape.elements.size()>1){ // Kurbo needs at least 2 points not to panic
            kurboToy* toy = toys[currentToy];
            if(toy){
                toy->applyFX(shape, fxShape);
                shape.bChanged = false;
            }
        }
    }

}

void ofApp::draw() {

    int textY = 50;
    ofClear(255,255,255);

    //ofDrawBitmapStringHighlight(ofToString(ofGetFrameRate()), 50, textY, ofColor(0,0,0,100), ofColor(255,255,255));
    //textY+=30;

    if(bShowHelp){
        ofDrawBitmapStringHighlight("Gui Toggles    : H=Help, I=Info", 50, textY, ofColor(0,0,0,100), ofColor(255,255,255));
        textY+=30;
        ofDrawBitmapStringHighlight("Render Toggles : A=Animate, N=Numbers", 50, textY, ofColor(0,0,0,100), ofColor(255,255,255));
        textY+=30;
        ofDrawBitmapStringHighlight("Commands       : D=Dump-SVG, PGDN=RegenerateShape", 50, textY, ofColor(0,0,0,100), ofColor(255,255,255));
        textY+=30;
        ofDrawBitmapStringHighlight("Use arrows to change toy. (--> <--)", 50, textY, ofColor(0,0,0,100), ofColor(255,255,255));
        textY+=30;
    }

    glm::vec2 infoTextPos(ofGetWidth()-50-200, 50);
    if(bShowInfo){
        ofDrawBitmapStringHighlight("Original Shape:", infoTextPos.x, infoTextPos.y, ofColor(0,0,0,100), ofColor(255,255,255));
        infoTextPos.y+=30;
        ofDrawBitmapStringHighlight(ofToString("Segments: ")+ofToString(shape.elements.size()), infoTextPos.x, infoTextPos.y, ofColor(0,0,0,100), ofColor(255,255,255));
        infoTextPos.y+=50;
        ofDrawBitmapStringHighlight("Transformed Shape:", infoTextPos.x, infoTextPos.y, ofColor(0,0,0,100), ofColor(255,255,255));
        infoTextPos.y+=30;
        ofDrawBitmapStringHighlight(ofToString("Segments: ")+ofToString(fxShape.elements.size()), infoTextPos.x, infoTextPos.y, ofColor(0,0,0,100), ofColor(255,255,255));
    }

    // Draw original
    shape.draw();

    // Draw FX shape
    //fxShape.draw(true, ofColor(255, 100, 100), ofColor(255, 200, 200));

    // // Shape editing (from ofxBezierRs)
    // // Preview bezier creation
    // if(clickedPos.x!=0 && clickedPos.y!=0){
    //     glm::vec2 offset;
    //     offset = {clickedPos.x-bezierPreview.x, clickedPos.y-bezierPreview.y};

    //     bezrsBezierHandle newBezier;
    //     newBezier.pos.x=clickedPos.x;
    //     newBezier.pos.y=clickedPos.y;
    //     newBezier.in_bez.x=clickedPos.x+offset.x;
    //     newBezier.in_bez.y=clickedPos.y+offset.y;
    //     newBezier.out_bez.x=clickedPos.x+offset.x*-1.f;
    //     newBezier.out_bez.y=clickedPos.y+offset.y*-1.f;

    //     // Draw shape preview
    //     ofNoFill();
    //     if(shape.beziers.size()>0){
    //         ofSetColor(ofColor::black);
    //         bezrsBezierHandle* bhPrev = &*shape.beziers.rbegin();
    //         bezrsBezierHandle* bhFirst = &*shape.beziers.begin();

    //         ofDrawBezier(bhPrev->pos.x, bhPrev->pos.y, bhPrev->out_bez.x, bhPrev->out_bez.y, newBezier.in_bez.x, newBezier.in_bez.y, newBezier.pos.x, newBezier.pos.y);
    //         ofDrawBezier(newBezier.pos.x, newBezier.pos.y, newBezier.out_bez.x, newBezier.out_bez.y, bhFirst->in_bez.x, bhFirst->in_bez.y, bhFirst->pos.x, bhFirst->pos.y);
    //     }

    //     // Draw bezier handle preview
    //     ofSetColor(ofColor::blue);
    //     ofDrawLine(newBezier.pos.x, newBezier.pos.y, newBezier.out_bez.x, newBezier.out_bez.y);
    //     ofDrawLine(newBezier.pos.x, newBezier.pos.y, newBezier.in_bez.x, newBezier.in_bez.y);
    // }

    // Draw transformed shape
    if(fxShape.elements.size()>1){
        fxShape.draw(true, ofColor::red);
    }

    // Show Toys
    kurboToy* toy = toys[currentToy];

    if(toy){
        int textX = 50;
        static const int fontSize = 8; // from ofDrawBitmapStringHighlight SRC
        static const int padding = 10;
        static const int toyOffset = fontSize * 7 + padding; // 7 = strlen(string_below)
        ofDrawBitmapStringHighlight(ofToString("Toys : "), textX, textY, ofColor(ofColor::black, 100));
        textX += toyOffset;
        for(auto* t : toys){
            int textWidth = fontSize * ofToString(t->name_cstr()).length() + padding;
            if(textX+textWidth > 600){ // Start new line ?
                textX = 50 + toyOffset;
                textY+= 20 + padding*.5;
            }
            ofDrawBitmapStringHighlight(t->name_cstr(), textX, textY, ofColor(ofColor::black, t==toy?255:100));
            textX += textWidth;
        }
        textY+=30;

        // Draw toy params
        if(bShowInfo) toy->drawParams(fxShape);
    }
}

void ofApp::keyPressed(int key) {
    // Handle Shortcuts
    if(key=='h' || key=='H'){
        // Toggle help
        bShowHelp = !bShowHelp;
    }
    else if(key=='i' || key=='I'){
        // Toggle help
        bShowInfo = !bShowInfo;
    }
    else if(key=='a' || key=='A'){
        // Toggle help
        bAnimate = !bAnimate;
    }
    else if(key=='n' || key=='N'){
        // Toggle help
        kurboShape::bShowNumbers = !kurboShape::bShowNumbers;
    }
    else if(key==OF_KEY_PAGE_DOWN){
        generateNewShape(!ofGetKeyPressed(OF_KEY_SHIFT));
    }
    else if (key == 'd' || key == 'D') {
        // Output current data to console
        auto elementsToSvgPath = [](const std::vector<KurboPathEl>& elements) -> std::string {
            std::string svg = "d=\"";
            for (const auto& el : elements) {
                switch (el.tag) {
                    case KurboPathElType::MoveTo:
                        svg += "M " + ofToString(el.p0.x) + " " + ofToString(el.p0.y) + " ";
                        break;
                    case KurboPathElType::LineTo:
                        svg += "L " + ofToString(el.p0.x) + " " + ofToString(el.p0.y) + " ";
                        break;
                    case KurboPathElType::QuadTo:
                        // Q takes 1 control point (p0) and 1 end point (p1)
                        svg += "Q " + ofToString(el.p0.x) + " " + ofToString(el.p0.y) + " " + ofToString(el.p1.x) + " " + ofToString(el.p1.y) + " ";
                        break;
                    case KurboPathElType::CurveTo:
                        // C takes 2 control points (p0, p1) and 1 end point (p2)
                        svg += "C " + ofToString(el.p0.x) + " " + ofToString(el.p0.y) + " " + ofToString(el.p1.x) + " " + ofToString(el.p1.y) + " " + ofToString(el.p2.x) + " " + ofToString(el.p2.y) + " ";
                        break;
                    case KurboPathElType::ClosePath:
                        svg += "Z ";
                        break;
                }
            }
            svg += "\"";
            return svg;
        };

        ofLogNotice("ofApp::keyPressed") << "--- ORIGINAL SHAPE DATA ---";
        ofLogNotice() << "<path " << elementsToSvgPath(shape.elements) << " />";
        
        ofLogNotice("ofApp::keyPressed") << "--- TRANSFORMED SHAPE DATA ---";
        ofLogNotice() << "<path " << elementsToSvgPath(fxShape.elements) << " />";
    }

    // Handle toy slider
    else if(key==OF_KEY_DOWN || key==OF_KEY_LEFT || key==OF_KEY_UP || key==OF_KEY_RIGHT){
        int dir = (key==OF_KEY_LEFT) ? -1 : 1;
        if(toys.size()>0){
            if(currentToy==0 && dir<0) currentToy = toys.size()-1;
            else currentToy = (currentToy+dir)%toys.size();
        }
        else currentToy = 0;
        shape.bChanged = true;
    }

    // Handle shape editing
    else if(key==OF_KEY_BACKSPACE){
        if(shape.elements.size()>0){
            shape.elements.pop_back();
            shape.bChanged = true;
        }
    }
}

void ofApp::generateNewShape(bool random) {
    shape.elements.clear();
    shape.bChanged = true;

    float w = ofGetWidth(), h = ofGetHeight();
    float cx = w * 0.5, cy = h * 0.5, r = std::min(w, h) * 0.3;
    
    // Fixed shapes
    if (!random){
        shape.elements.push_back({KurboPathElType::MoveTo, {cx, cy - r}, {0,0}, {0,0}});
        shape.elements.push_back({KurboPathElType::CurveTo, {cx + r, cy - r}, {cx + r, cy + r * 0.5}, {cx, cy + r}});
        shape.elements.push_back({KurboPathElType::CurveTo, {cx - r, cy + r * 0.5}, {cx - r, cy - r}, {cx, cy - r}});
        shape.elements.push_back({KurboPathElType::ClosePath, {0,0}, {0,0}, {0,0}});
        
        shape.elements.push_back({KurboPathElType::MoveTo, {cx - r * 1.5, cy + r * 1.2}, {0,0}, {0,0}});
        shape.elements.push_back({KurboPathElType::CurveTo, {cx - r * 0.5, cy + r * 1.5}, {cx + r * 0.5, cy + r * 0.9}, {cx + r * 1.5, cy + r * 1.2}});
    }
    // Random "star-like" shape (with jitter).
    else {
        shape.elements.clear();
        
        glm::vec2 center = {ofGetWidth() * 0.5f, ofGetHeight() * 0.5f};
        const float numPts = 10.f;
        const float radius = 160.f;
        const float variance = 8.f;
        const float bezierSize = 35.f;
        const float bezierVariance = 0.1f * TWO_PI;
        
        // Temporary variables to hold the first point's data for closing the loop
        glm::vec2 firstAnchor = {0.f, 0.f};
        glm::vec2 firstInHandle = {0.f, 0.f};
        glm::vec2 currentOutHandle = {0.f, 0.f};
        
        for (unsigned int i = 0; i < (unsigned int)numPts; i++) {
            bool pair = i % 2;
            float angle = (i / numPts) * TWO_PI;
            
            // Alternating radius (half radius, full radius) + random variance
            glm::vec2 ptCenter = center + glm::vec2(
                cos(angle) * radius * (0.5f + 0.5f * pair),
                sin(angle) * radius * (0.5f + 0.5f * pair)
            );
            ptCenter += glm::vec2(ofRandom(0.f, variance), ofRandom(0.f, variance));
            
            // Randomize handle configuration (0 = straight, 1 = symmetric, 2 = asymmetric)
            int rand = roundf(ofRandom(-0.5f, 2.4f)); 
            
            glm::vec2 radial = glm::vec2(cos(angle), sin(angle));
            // Tangent vector perpendicular to radial, with random variance
            float rotAngle = -glm::half_pi<float>() + ofRandom(-bezierVariance, bezierVariance);
            glm::vec2 tan = glm::rotate(radial, rotAngle);
            
            glm::vec2 bhInOffset = {0.f, 0.f};
            glm::vec2 bhOutOffset = {0.f, 0.f};
            
            if (rand == 1 || rand == 2) {
                float size = ofRandom(bezierSize - variance, bezierSize + variance);
                bhInOffset = tan * size;
                bhOutOffset = bhInOffset * glm::vec2(-1.f, -1.f); // Opposite direction
                
                if (rand == 2) {
                    // Asymmetric: randomize the out-handle direction independently
                    float size2 = ofRandom(bezierSize - variance, bezierSize + variance);
                    float rotAngle2 = glm::half_pi<float>() + ofRandom(-bezierVariance, bezierVariance);
                    bhOutOffset = size2 * glm::rotate(radial, rotAngle2);
                }
            }
            
            glm::vec2 currentInHandle = ptCenter + bhInOffset;
            glm::vec2 nextOutHandle = ptCenter + bhOutOffset;
            
            if (i == 0) {
                // First point: MoveTo and save data for the final closing segment
                firstAnchor = ptCenter;
                firstInHandle = currentInHandle;
                currentOutHandle = nextOutHandle;
                
                shape.elements.push_back({
                    KurboPathElType::MoveTo, 
                    to_kurboPos(firstAnchor), 
                    {0.f, 0.f}, {0.f, 0.f}
                });
            } else {
                // Subsequent points: directly emit a CurveTo segment
                // p0 = previous point's out-handle, p1 = current point's in-handle, p2 = current point's anchor
                shape.elements.push_back({
                    KurboPathElType::CurveTo,
                    to_kurboPos(currentOutHandle),
                    to_kurboPos(currentInHandle),
                    to_kurboPos(ptCenter)
                });
                currentOutHandle = nextOutHandle;
            }
        }
        
        // Close the loop: emit the final CurveTo segment connecting back to the first point
        shape.elements.push_back({
            KurboPathElType::CurveTo,
            to_kurboPos(currentOutHandle), // Last point's out-handle
            to_kurboPos(firstInHandle),    // First point's in-handle
            to_kurboPos(firstAnchor)       // First point's anchor
        });
        
        // Explicitly close the path
        shape.elements.push_back({
            KurboPathElType::ClosePath, 
            {0.f, 0.f}, {0.f, 0.f}, {0.f, 0.f}
        });
        
        shape.bChanged = true;
    }
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
