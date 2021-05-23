//
// Created by Caelum van Ispelen on 5/22/21.
//

#include "era.h"

namespace rip {
    const char *eraID(Era era) {
        switch (era) {
            case Ancient:
                return "ancient";
            case Classical:
                return "classical";
            case Medieval:
                return "medieval";
            case Renaissance:
                return "renaissance";
            case Industrial:
                return "industrial";
            case Modern:
                return "modern";
            case Future:
                return "future";
        }
    }

    Era eraFromID(const std::string &id) {
        if (id == "ancient") return Era::Ancient;
        else if (id == "classical") return Era::Classical;
        else if (id == "medieval") return Era::Medieval;
        else if (id == "renaissance") return Era::Renaissance;
        else if (id == "industrial") return Era::Industrial;
        else if (id == "modern") return Era::Modern;
        else return Era::Future;
    }
}
