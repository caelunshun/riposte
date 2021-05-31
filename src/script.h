//
// Created by Caelum van Ispelen on 5/31/21.
//

#ifndef RIPOSTE_SCRIPT_H
#define RIPOSTE_SCRIPT_H

#include <memory>
#include "assets.h"

namespace rip {
    struct ScriptImpl;
    class Player;
    class Game;

    // Intentionally empty.
    struct ScriptAsset : public Asset {};

    // Manages Lua scripting.
    class ScriptEngine {
    public:
        std::unique_ptr<ScriptImpl> impl;

        ScriptEngine();
        ~ScriptEngine();
        ScriptEngine(ScriptEngine &&other);
        ScriptEngine(const ScriptEngine &other) = delete;

        void setGame(Game *game);

        void onWarDeclared(Player &declarer, Player &declared);
    };

    class ScriptLoader : public AssetLoader {
        std::shared_ptr<ScriptEngine> engine;
    public:
        ScriptLoader(const std::shared_ptr<ScriptEngine> &engine);

        std::shared_ptr<Asset> loadAsset(const std::string &data) override;
    };
}

#endif //RIPOSTE_SCRIPT_H
