//
// Created by Caelum van Ispelen on 5/14/21.
//

#ifndef RIPOSTE_UI_H
#define RIPOSTE_UI_H

#include <nuklear.h>

namespace rip {
    class Ui {
        nk_context *nk;

    public:
        Ui() : nk(nullptr) {}

        nk_context *getNk() const { return nk; }
    };
}


#endif //RIPOSTE_UI_H
