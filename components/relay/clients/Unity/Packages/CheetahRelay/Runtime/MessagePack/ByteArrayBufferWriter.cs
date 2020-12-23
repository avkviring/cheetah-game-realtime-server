using System;
using System.Buffers;

namespace CheetahRelay.Runtime.Client.Codec
{
    public class ByteArrayBufferWriter : IBufferWriter<byte>
    {
        private byte[] _buffer;
        public int Count;

        public ByteArrayBufferWriter(byte[] buffer)
        {
            _buffer = buffer;
            Count = 0;
        }

        public void Clear()
        {
            Count = 0;
        }

        public void Advance(int count)
        {
            Count += count;
        }

        public Memory<byte> GetMemory(int sizeHint = 0)
        {
            return new Memory<byte>(_buffer);
        }

        public Span<byte> GetSpan(int sizeHint = 0)
        {
            return new Span<byte>(_buffer);
        }
    }
}