using System;
using Games.Cheetah.Client.Codec;

namespace Games.Cheetah.Client.Editor.Generator.Fields.Array.Exceptions
{
    public class MissingArraySizeFieldAttributeException : Exception
    {
        public MissingArraySizeFieldAttributeException() : base($"Missing {nameof(ArraySizeField)} attribute.")
        {
        }
    }
}