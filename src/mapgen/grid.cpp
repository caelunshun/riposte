//
// Created by Caelum van Ispelen on 8/30/21.
//

#include "grid.h"

#include <string>

rip::mapgen::OutOfBoundsException::OutOfBoundsException(int width, int height, int x, int y) : width(width),
                                                                                               height(height), x(x),
                                                                                               y(y) {
    message = "grid index out of bounds: (" + std::to_string(x) + ", " + std::to_string(y) + ")";
}

const char *rip::mapgen::OutOfBoundsException::what() const noexcept {
    return message.c_str();
}
