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

SoundHandle *rodio_create_sound(const uint8_t *data, size_t length);

bool rodio_is_sound_done(InstanceHandle *sound);

OutputDevice *rodio_new();

InstanceHandle *rodio_start_sound(OutputDevice *device, const SoundHandle *sound);

void rodio_stop_sound(InstanceHandle *sound);

} // extern "C"
