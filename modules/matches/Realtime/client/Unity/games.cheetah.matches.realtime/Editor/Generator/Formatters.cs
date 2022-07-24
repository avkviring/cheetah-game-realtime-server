using System;
using System.Collections.Generic;
using Cheetah.Matches.Relay.Codec.Formatter;
using JetBrains.Annotations;

namespace Cheetah.Matches.Relay.Editor.Generator
{
    public class Formatters
    {
        private readonly Dictionary<Type, Type> formatters = new();
        private readonly HashSet<Type> supportFixedArray = new();

        private readonly Dictionary<Type, Type> variableSizeFormatters = new();

        public Formatters()
        {
            formatters.Add(typeof(bool), typeof(BoolFormatter));
            formatters.Add(typeof(byte), typeof(ByteFormatter));
            formatters.Add(typeof(short), typeof(ShortFormatter));
            formatters.Add(typeof(ushort), typeof(UShortFormatter));
            formatters.Add(typeof(int), typeof(IntFormatter));
            formatters.Add(typeof(uint), typeof(UIntFormatter));
            formatters.Add(typeof(long), typeof(LongFormatter));
            formatters.Add(typeof(ulong), typeof(ULongFormatter));
            formatters.Add(typeof(float), typeof(FloatFormatter));
            formatters.Add(typeof(double), typeof(DoubleFormatter));
            formatters.Add(typeof(string), typeof(StringFormatter));

            supportFixedArray.Add(typeof(bool));
            supportFixedArray.Add(typeof(byte));
            supportFixedArray.Add(typeof(short));
            supportFixedArray.Add(typeof(ushort));
            supportFixedArray.Add(typeof(int));
            supportFixedArray.Add(typeof(uint));
            supportFixedArray.Add(typeof(long));
            supportFixedArray.Add(typeof(ulong));
            supportFixedArray.Add(typeof(float));
            supportFixedArray.Add(typeof(double));

            variableSizeFormatters.Add(typeof(uint), typeof(VariableSizeUIntFormatter));
            variableSizeFormatters.Add(typeof(int), typeof(VariableSizeIntFormatter));
            variableSizeFormatters.Add(typeof(ulong), typeof(VariableSizeULongFormatter));
            variableSizeFormatters.Add(typeof(long), typeof(VariableSizeLongFormatter));
        }


        public bool IsSupportFixedArray(Type type)
        {
            return supportFixedArray.Contains(type);
        }

        [CanBeNull]
        public string GetFormatterInstanceName(Type type)
        {
            return formatters.TryGetValue(type, out var formatter) ? formatter.Name + ".Instance" : null;
        }

        [CanBeNull]
        public string GetVariableSizeFormatterInstanceName(Type type)
        {
            return variableSizeFormatters.TryGetValue(type, out var formatter) ? formatter.Name + ".Instance" : null;
        }
    }
}