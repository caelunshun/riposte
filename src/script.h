//
// Created by Caelum van Ispelen on 5/31/21.
//

#ifndef RIPOSTE_SCRIPT_H
#define RIPOSTE_SCRIPT_H

#include <memory>
#include <glm/vec2.hpp>
#include "assets.h"
#include <nanovg.h>

namespace rip {
    struct ScriptImpl;
    class Player;
    class Game;
    class Hud;

    // Intentionally empty.
    struct ScriptAsset : public Asset {};

    struct Canvas {
        NVGcontext *vg;

        Canvas(NVGcontext *vg) : vg(vg) {}
    };

    // Manages Lua scripting.
    class ScriptEngine {
    public:
        std::unique_ptr<ScriptImpl> impl;

        ScriptEngine();
        ~ScriptEngine();
        ScriptEngine(ScriptEngine &&other);
        ScriptEngine(const ScriptEngine &other) = delete;

        void registerHudBindings(std::shared_ptr<Hud> hud);

        void setGame(Game *game);

        void onWarDeclared(Player &declarer, Player &declared);
        void onDialogueOpened(Player &with);

        void onPosClicked(glm::uvec2 pos);
        void onKeyPressed(int key);
        void onTurnEnd();
    };

    class ScriptLoader : public AssetLoader {
        std::shared_ptr<ScriptEngine> engine;
    public:
        ScriptLoader(const std::shared_ptr<ScriptEngine> &engine);

        std::shared_ptr<Asset> loadAsset(const std::string &data) override;
    };
}

#endif //RIPOSTE_SCRIPT_H
