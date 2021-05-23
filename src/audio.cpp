//
// Created by Caelum van Ispelen on 5/22/21.
//

#include <regex>
#include <iostream>
#include "audio.h"
#include "game.h"

namespace rip {

    std::shared_ptr<Asset> AudioLoader::loadAsset(const std::string &data) {
        auto handle = rodio_create_sound(reinterpret_cast<const uint8_t *>(data.data()), data.size());
        return std::make_shared<SoundAsset>(handle);
    }

    SoundAsset::SoundAsset(SoundHandle *handle) : handle(handle) {}

    AudioManager::AudioManager() {
        rodio = rodio_new();
    }

    void AudioManager::addSounds(const Assets &assets) {
        std::regex eraRegex("music/(.+)/(.+)");
        for (const auto &entry : assets.getAllWithIDs<SoundAsset>()) {
            auto &id = entry.first;
            auto &sound = entry.second;

            std::smatch match;
            if (std::regex_search(id.begin(), id.end(), match, eraRegex)) {
                auto era = eraFromID(match[1]);
                if (!eraMusic.contains(era)) eraMusic[era] = FairPicker<std::shared_ptr<SoundAsset>>();
                eraMusic[era].addChoice(sound);
                std::cout << "[audio] Detected music '" << match[2] << "' for era " << eraID(era) << std::endl;
            }
        }
    }

    void AudioManager::updateEraMusic(const Game &game) {
        auto currentEra = game.getEra();
        if (!currentMusic.has_value() || currentMusicEra != currentEra
            || rodio_is_sound_done(*currentMusic)) {
            if (currentMusic.has_value()) {
                rodio_stop_sound(*currentMusic);
            }

            auto sound = eraMusic[currentEra].pickNext();

            currentMusic = rodio_start_sound(rodio, sound->handle);
            currentMusicEra = currentEra;
        }
    }

    void AudioManager::update(const Game &game) {
        updateEraMusic(game);
    }
}
