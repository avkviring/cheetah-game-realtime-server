using System;
using Cheetah.Matches.Relay.Codec;

namespace Cheetah.Matches.Relay.Editor.Generator.Fields.Array.Exceptions
{
    public class MissingArraySizeFieldAttributeException : Exception
    {
        public MissingArraySizeFieldAttributeException() : base($"Missing {nameof(ArraySizeField)} attribute.")
        {
        }
    }
}