using Cheetah.Matches.Realtime.Codec.Formatter;

namespace Cheetah.Matches.Realtime.Tests.Codec.Formatter
{
    public class TestByteAbstractFormatter : AbstractUnmanagedFormatterTest<byte, ByteFormatter>
    {
        protected override byte[] GetValues()
        {
            return new byte[] { 0, byte.MaxValue };
        }
    }
}