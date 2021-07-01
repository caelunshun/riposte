//
// Created by Caelum van Ispelen on 6/30/21.
//

#include "slot_map.h"

const char *rip::InvalidIDException::what() const noexcept {
    return "invalidated ID used as slotmap index";
}

const char *rip::TooManyItemsException::what() const noexcept {
    return "slotmap contains more than 2^16 elements";
}
