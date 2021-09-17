//
// Created by Caelum van Ispelen on 9/1/21.
//

#include <FastNoise/FastNoise.h>

#include "terrain.h"

namespace rip::mapgen {
    Grid<Tile> DefaultTerrainGenerator::generateTerrain(const Grid<LandCell> &landGrid, Rng &rng) {
        std::cerr << "T" << 11 << std::endl;
        std::cerr << landGrid.getWidth() << ", " << landGrid.getHeight() << std::endl;
        Grid<Tile> tileGrid(landGrid.getWidth(), landGrid.getHeight(), Tile());

        std::cerr << "T" << 13 << std::endl;
        
        // Use noise to determine terrain types.
        auto cellular = FastNoise::New<FastNoise::CellularValue>();
        auto terrainNoise = FastNoise::New<FastNoise::FractalFBm>();
        terrainNoise->SetSource(cellular);
        auto treeNoise = FastNoise::New<FastNoise::FractalFBm>();
        treeNoise->SetSource(cellular);

        std::vector<float> terrainNoiseOutput(tileGrid.getWidth() * tileGrid.getHeight());
        terrainNoise->GenUniformGrid2D(terrainNoiseOutput.data(), 0, 0, tileGrid.getWidth(), tileGrid.getHeight(), 0.5, rng.u32(0, 0xFFFFFFFF));

        std::vector<float> treeNoiseOutput(tileGrid.getWidth() * tileGrid.getHeight());
        treeNoise->GenUniformGrid2D(treeNoiseOutput.data(), 0, 0, tileGrid.getWidth(), tileGrid.getHeight(), 5.0f, rng.u32(0, 0xFFFFFFFF));

        auto simplex = FastNoise::New<FastNoise::Simplex>();
        auto hillNoise = FastNoise::New<FastNoise::FractalFBm>();
        hillNoise->SetSource(simplex);

        std::vector<float> hillNoiseOutput(tileGrid.getWidth() * tileGrid.getHeight());
        hillNoise->GenUniformGrid2D(hillNoiseOutput.data(), 0, 0, tileGrid.getWidth(), tileGrid.getHeight(), 5.0f, rng.u32(0, 0xFFFFFFFF));

        std::cerr << "T" << 35 << std::endl;

        for (int x = 0; x < tileGrid.getWidth(); x++) {
            for (int y = 0; y < tileGrid.getHeight(); y++) {
                auto noiseIndex = x + y * tileGrid.getWidth();
                Terrain t;
                if (landGrid.get(x, y) == LandCell::Land) {
                    auto choice = terrainNoiseOutput[noiseIndex];
                    if (choice < -0.1) {
                        t = Terrain::Grassland;
                    } else if (choice < 0.4) {
                        t = Terrain::Plains;
                    } else {
                        t = Terrain::Desert;
                    }
                } else {
                    t = Terrain::Ocean;
                }

                tileGrid.get(x, y).setTerrain(t);

                if (t != Terrain::Ocean) {
                    // Hills
                    if (hillNoiseOutput[noiseIndex] > 0.2) {
                        tileGrid.get(x, y).setHilled(true);
                    }

                    // Forest
                    if (t != Terrain::Desert) {
                        if (treeNoiseOutput[noiseIndex] < 0.3) {
                            tileGrid.get(x, y).setForested(true);
                        }
                    }
                }
            }
        }

        std::cerr << "T" << 72 << std::endl;

        return tileGrid;
    }
}
