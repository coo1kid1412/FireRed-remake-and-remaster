#import <AVFAudio/AVFAudio.h>
#import <Foundation/Foundation.h>

void *firered_music_play(const char *path) {
    @autoreleasepool {
        NSURL *url = [NSURL fileURLWithFileSystemRepresentation:path
                                                   isDirectory:NO
                                                 relativeToURL:nil];
        AVAudioPlayer *player = [[AVAudioPlayer alloc] initWithContentsOfURL:url error:NULL];
        player.numberOfLoops = -1;
        if (![player play]) {
            return NULL;
        }
        return (__bridge_retained void *)player;
    }
}

void firered_music_stop(void *handle) {
    @autoreleasepool {
        AVAudioPlayer *player = (__bridge_transfer AVAudioPlayer *)handle;
        [player stop];
    }
}
