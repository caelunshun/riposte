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

    void AudioManager::addSounds(std::shared_ptr<Assets> assets) {
        this->assets = assets;
        std::regex eraRegex("music/(.+)/(.+)");
        for (const auto &entry : assets->getAllWithIDs<SoundAsset>()) {
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

            currentMusic = playSound(*sound);
            if (currentMusic == nullptr) currentMusic = {};
            currentMusicEra = currentEra;
        }
    }

    void AudioManager::playSound(const std::string &id) {
        auto sound = std::dynamic_pointer_cast<SoundAsset>(assets->get(id));
        playSound(*sound);
    }

    void AudioManager::update(const Game &game) {
        updateEraMusic(game);

        // check for completed sounds
        if (!playingSounds.empty()) {
            for (int i = playingSounds.size() - 1; i >= 0; i--) {
                if (rodio_is_sound_done(playingSounds[i])) {
                    playingSounds.erase(playingSounds.begin() + i);
                }
            }
        }
    }

    InstanceHandle *AudioManager::playSound(const SoundAsset &sound) {
        const auto maxSounds = 16;
        if (playingSounds.size() > maxSounds) {
            return nullptr;
        }

        auto *instance = rodio_start_sound(rodio, sound.handle);
        playingSounds.push_back(instance);
        return instance;
    }
}
