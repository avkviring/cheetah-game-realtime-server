using System.Collections.Generic;
using Cheetah.Matches.Realtime.Codec;
using Cheetah.Matches.Realtime.Editor.Generator;
using Cheetah.Matches.Realtime.Editor.Generator.Fields.Array;
using Cheetah.Matches.Realtime.Editor.Generator.Fields.Array.Exceptions;
using NUnit.Framework;

namespace Cheetah.Matches.Realtime.Tests.Generator.Fields.Array
{
    public class TestReferencedArrayFieldCodecGenerator
    {
        [Test]
        public void ShouldNullIfNotArray()
        {
            Assert.Null(CodecArrayFieldGenerator.Create(
                new CodecsImporter("test"),
                new FieldInfoStubBuilder<int>("name").Build(),
                new HashSet<string>(),
                new HashSet<string>()));
        }

        [Test]
        public void ShouldThrowIfArraySizeFieldAttributeNotPresent()
        {
            Assert.Throws<MissingArraySizeFieldAttributeException>(() =>
                CodecArrayFieldGenerator.Create(
                    new CodecsImporter("test"),
                    new FieldInfoStubBuilder<int[]>("name").Build(),
                    new HashSet<string>(),
                    new HashSet<string>()));
        }

        [Test]
        public void ShouldFailWhenMissingArraySizeField()
        {
            Assert.Throws<MissingArraySizeFieldException>(() =>
            {
                CodecArrayFieldGenerator.Create(
                    new CodecsImporter("test"),
                    CreateFieldInfoAccessor(),
                    new HashSet<string>(),
                    new HashSet<string>());
            });
        }

        [Test]
        public void ShouldFailWhenIncorrectOrderArraySizeField()
        {
            Assert.Throws<IncorrectOrderArraySizeFieldException>(() =>
            {
                CodecArrayFieldGenerator.Create(
                    new CodecsImporter("test"),
                    CreateFieldInfoAccessor(),
                    new HashSet<string>(),
                    new HashSet<string> { "size" });
            });
        }

        [Test]
        public void ShouldCreate()
        {
            var generator = CodecArrayFieldGenerator.Create(
                new CodecsImporter("test"),
                CreateFieldInfoAccessor(),
                new HashSet<string> { "size" },
                new HashSet<string> { "size" });
            Assert.NotNull(generator);
        }

        private static FieldInfoAccessorStub CreateFieldInfoAccessor()
        {
            var builder = new FieldInfoStubBuilder<int[]>("name");
            builder.AddAttribute(new ArraySizeField("size"));
            var fieldInfoAccessorStub = builder.Build();
            return fieldInfoAccessorStub;
        }
    }
}