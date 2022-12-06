using Games.Cheetah.Client.Codec.Formatter;

namespace Games.Cheetah.Client.Tests.Codec.Formatter
{
    public class TestByteAbstractFormatter : AbstractUnmanagedFormatterTest<byte, ByteFormatter>
    {
        protected override byte[] GetValues()
        {
            return new byte[] { 0, byte.MaxValue };
        }
    }
}