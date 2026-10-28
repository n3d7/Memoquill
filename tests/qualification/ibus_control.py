"""Select and verify a real engine on this test session's private IBus bus."""
import json
import sys
import gi

gi.require_version('IBus', '1.0')
from gi.repository import IBus

def select_engine(name):
    IBus.init()
    bus = IBus.Bus()
    if not bus.is_connected():
        raise RuntimeError('Private IBus bus is not connected')
    if not bus.set_global_engine(name):
        raise RuntimeError(f'IBus rejected engine {name}')
    current = bus.get_global_engine()
    if current is None or current.get_name() != name:
        raise RuntimeError(f'IBus did not activate {name}')
    return current.get_name()

if __name__ == '__main__':
    print(json.dumps({'activeEngine': select_engine(sys.argv[1])}))
