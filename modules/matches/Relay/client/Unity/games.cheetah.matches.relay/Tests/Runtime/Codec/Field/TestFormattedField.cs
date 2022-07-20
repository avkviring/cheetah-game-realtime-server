using Cheetah.Matches.Relay.Codec;
using Cheetah.Matches.Relay.Tests.Runtime.Codec.Types;
using NUnit.Framework;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Field
{
    public class TestFormattedField : AbstractFieldTest<TestFormattedField.Structure>
    {
        [GenerateCodec]
        public struct Structure
        {
            public bool boolValue;
            public byte byteValue;
            public short shortValue;
            public ushort ushortValue;
            public uint uintValue;
            public int intValue;
            public long longValue;
            public float floatValue;
            public double doubleValue;
            public string name;
        }

        protected override Structure GetSource()
        {
            return new Structure
            {
                boolValue = true,
                intValue = 55,
                byteValue = 1,
                doubleValue = 1.1,
                floatValue = (float)1.2,
                longValue = -10000,
                shortValue = -10,
                ushortValue = 55,
                uintValue = 123123,
                name = "Петя"
            };
        }

        protected override void CheckResult(Structure source, Structure result)
        {
            Assert.AreEqual(source.boolValue, result.boolValue);
            Assert.AreEqual(source.byteValue, result.byteValue);
            Assert.AreEqual(source.shortValue, result.shortValue);
            Assert.AreEqual(source.ushortValue, result.ushortValue);
            Assert.AreEqual(source.uintValue, result.uintValue);
            Assert.AreEqual(source.intValue, result.intValue);
            Assert.AreEqual(source.longValue, result.longValue);
            Assert.AreEqual(source.floatValue, result.floatValue);
            Assert.AreEqual(source.doubleValue, result.doubleValue);
            Assert.AreEqual(source.name, result.name);
        }
    }
}