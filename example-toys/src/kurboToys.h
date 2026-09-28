#pragma once

#include "ofxKurbo.h"
#include "kurboShape.h"
#include <vector>
#include <string>

class kurboToy {
    const std::string name;
public:
    const char* name_cstr();
    kurboToy(std::string _name="Unknown") : name(_name) {};
    virtual void applyFX(const kurboShape& _inShape, kurboShape& _outShape) = 0;
    virtual void drawParams(const kurboShape& _sh);
};

class offsetToy : public kurboToy {
public:
    offsetToy(std::string _name="Offset") : kurboToy(_name) {};
    void applyFX(const kurboShape& _inShape, kurboShape& _outShape) override;
    void drawParams(const kurboShape& _sh) override;
protected:
    float offset = 0.f;
    kurboJoinType join = kurboJoinType::Bevel;
    void updateParams();
    unsigned int lastTime = 0;
};

class outlineToy : public offsetToy {
public:
    outlineToy() : offsetToy("Outline") {};
    void applyFX(const kurboShape& _inShape, kurboShape& _outShape) override;
    void drawParams(const kurboShape& _sh) override;
protected:
    //kurboShape outlineShapeBis;
};

class rotationToy : public kurboToy {
public:
    rotationToy() : kurboToy("Rotation") {};
    void applyFX(const kurboShape& _inShape, kurboShape& _outShape) override;
    void drawParams(const kurboShape& _sh) override;
protected:
    kurboPos center;
    float rotation = 0.f;
    void updateParams();
};

class reverseWindingToy : public kurboToy {
public:
    reverseWindingToy() : kurboToy("Reverse Winding") {};
    void applyFX(const kurboShape& _inShape, kurboShape& _outShape) override;
    void drawParams(const kurboShape& _sh) override;
    void updateParams();
    bool reversed = true;
};

class boundingBoxToy : public kurboToy {
public:
    boundingBoxToy() : kurboToy("Bounding Box") {};
    void applyFX(const kurboShape& _inShape, kurboShape& _outShape) override;
    void drawParams(const kurboShape& _sh) override;
    void updateParams();
protected:
    kurboRect bb;
};

class hitTestToy : public kurboToy {
public:
    hitTestToy() : kurboToy("Hit test") {};
    void applyFX(const kurboShape& _inShape, kurboShape& _outShape) override;
    void drawParams(const kurboShape& _sh) override;
protected:
    glm::vec2 simPos = {0,0};
    const float simSpeed = 3;
    glm::vec2 simVec = {simSpeed, simSpeed};
    glm::vec2 mousePos = {0,0};
    bool simPosHit = false;
    bool mousePosHit = false;
    glm::vec2 mouseProjection, simProjection;
    void updateParams();
};

class inflectionsToy : public kurboToy {
public:
    inflectionsToy() : kurboToy("Inflections") {};
    void applyFX(const kurboShape& _inShape, kurboShape& _outShape) override;
    void drawParams(const kurboShape& _sh) override;
protected:
    std::vector<glm::vec2> inflections;
    std::vector<glm::vec2> local_extremas;
    void updateParams();
};

class evaluateToy : public kurboToy {
public:
    evaluateToy() : kurboToy("Evaluate TValue") {};
    void applyFX(const kurboShape& _inShape, kurboShape& _outShape) override;
    void drawParams(const kurboShape& _sh) override;
protected:
    double tval = 0.;
    kurboEvalResult evalRes;
    bool bEuclidean = true; // true = arc-length, false = linear segment distribution
    void updateParams();
};

class selfIntersectToy : public offsetToy {
public:
    selfIntersectToy() : offsetToy("Offset Self Intersect") {};
    void applyFX(const kurboShape& _inShape, kurboShape& _outShape) override;
    void drawParams(const kurboShape& _sh) override;
protected:
    std::vector<glm::vec2> selfIntersects;
    std::vector<double> floatsVec;
    void updateParams();
};

class booleanToy : public kurboToy {
public:
    booleanToy() : kurboToy("Boolean Ops") {};
    void applyFX(const kurboShape& _inShape, kurboShape& _outShape) override;
    void drawParams(const kurboShape& _sh) override;
protected:
    void updateParams();
    KurboBooleanOp booleanOp = KurboBooleanOp::Xor;
    float rotation = 0.f;
    bool bRotate = true;
    kurboRect bb;
    kurboPos bbCenter;
    kurboShape rotatedShape;
};
