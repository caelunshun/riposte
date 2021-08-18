#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>


struct InstanceHandle;

struct OutputDevice;

struct SoundHandle;


extern "C" {

const char *riposte_data_dir();

SoundHandle *rodio_create_sound(const uint8_t *data, size_t length);

void rodio_free_sound(InstanceHandle *sound);

bool rodio_is_sound_done(InstanceHandle *sound);

OutputDevice *rodio_new();

void rodio_sound_set_volume(InstanceHandle *sound, float volume);

InstanceHandle *rodio_start_sound(OutputDevice *device, const SoundHandle *sound, float volume);

void rodio_stop_sound(InstanceHandle *sound);

} // extern "C"
