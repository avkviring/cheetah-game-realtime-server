using System;
using System.Collections.Generic;
using System.Linq;

namespace Games.Cheetah.Client.Editor.DumpViewer.Sections.Objects.Query
{
    public class Parser
    {
        /// <summary>
        /// Преобразовать текстовый запрос в набор правил
        /// </summary>
        /// <param name="query"></param>
        /// <exception cref="ParserException">Ошибка синтаксиса</exception>
        /// <returns>
        /// </returns>
        public static Rule Parse(string query)
        {
            var queryWithoutSpaces = query.Replace(" ", "");
            if (ParseExpression(queryWithoutSpaces) is RuleTerm ruleTerm)
            {
                return ruleTerm.rule;
            }

            throw new ParserException("Internal exception: root rule is not RuleTerm");
        }

        private static Term ParseExpression(string query)
        {
            Console.Out.WriteLine("ParseExpression " + query);
            var remaining = query;
            var terms = new List<Term>();
            while (remaining.Length > 0)
            {
                var c = remaining.First();
                switch (c)
                {
                    case '(':
                    {
                        var rightBracket = IndexRightBracket(remaining);
                        var term = ParseExpression(remaining.Substring(1, rightBracket - 1));
                        terms.Add(term);
                        remaining = remaining.Remove(0, rightBracket);
                        break;
                    }
                    case ')':
                        remaining = remaining.Remove(0, 1);
                        break;
                    case '&':
                        remaining = remaining.Remove(0, 2);
                        terms.Add(new AndTerm());
                        break;
                    case '|':
                        remaining = remaining.Remove(0, 2);
                        terms.Add(new OrTerm());
                        break;
                    default:
                    {
                        var indexOfEndFieldRule = IndexOfEndFieldRule(remaining);
                        var item = remaining.Substring(0, indexOfEndFieldRule);
                        remaining = remaining.Remove(0, indexOfEndFieldRule);
                        var rule = ParseFieldRule(item);
                        terms.Add(rule);
                        break;
                    }
                }
            }

            var result = Reduce(terms);
            if (result.Count > 1)
            {
                throw new ParserException("Internal parser exception: more one rules after reduce");
            }

            if (result.Count == 0)
            {
                throw new ParserException("Internal parser exception: no rules");
            }

            return result.First();
        }

        private static List<Term> Reduce(List<Term> terms)
        {
            terms = Reduce(terms, current => current is AndTerm, (current, prev, next) =>
            {
                var prevRule = ((RuleTerm)prev).rule;
                var nextRule = ((RuleTerm)next).rule;
                var andRule = new AndRule(new List<Rule>());
                andRule.AddRule(prevRule);
                andRule.AddRule(nextRule);
                return new RuleTerm(andRule);
            });
            terms = Reduce(terms, current => current is OrTerm, (current, prev, next) =>
            {
                var prevRule = ((RuleTerm)prev).rule;
                var nextRule = ((RuleTerm)next).rule;
                var orRule = new OrRule(new List<Rule>());
                orRule.AddRule(prevRule);
                orRule.AddRule(nextRule);
                return new RuleTerm(orRule);
            });
            return terms;
        }

        private static int IndexOfEndFieldRule(string line)
        {
            var andIndex = line.IndexOf("&&", StringComparison.Ordinal);
            var orIndex = line.IndexOf("||", StringComparison.Ordinal);
            if (andIndex == -1 && orIndex == -1)
            {
                return line.Length;
            }

            if (andIndex == -1)
            {
                return orIndex;
            }

            if (orIndex == -1)
            {
                return andIndex;
            }

            return andIndex < orIndex ? andIndex : orIndex;
        }

        /// <summary>
        /// Ищем правую закрывающую скобку
        /// </summary>
        /// <param name="line"></param>
        /// <returns></returns>
        /// <exception cref="ParserException"></exception>
        private static int IndexRightBracket(string line)
        {
            var stack = 0;
            for (var i = 0; i < line.Length; i++)
            {
                var c = line[i];
                switch (c)
                {
                    case '(':
                        stack++;
                        break;
                    case ')':
                    {
                        stack--;
                        if (stack == 0)
                        {
                            return i;
                        }

                        break;
                    }
                }
            }

            throw new ParserException("Right bracket not found for " + line);
        }

        /// <summary>
        /// Раскрываем групповые Term (AndTerm, OrTerm)
        /// </summary>
        /// <param name="source"></param>
        /// <param name="shouldReduce"></param>
        /// <param name="doReduce"></param>
        /// <returns></returns>
        private static List<Term> Reduce(IEnumerable<Term> source, Func<Term, bool> shouldReduce, Func<Term, Term, Term, Term> doReduce)
        {
            var result = new List<Term>(source);
            var i = 0;
            while (i < result.Count)
            {
                var current = result[i];

                if (shouldReduce(current))
                {
                    var reduceResult = doReduce(current, result[i - 1], result[i + 1]);
                    result[i - 1] = reduceResult;
                    result.RemoveAt(i);
                    result.RemoveAt(i);
                }
                else
                {
                    i += 1;
                }
            }

            return result;
        }


        private static Term ParseFieldRule(string query)
        {
            var not = query.Contains("!=");
            var fields = query.Split(new[] { not ? "!=" : "=" }, StringSplitOptions.None);
            if (fields.Count() != 2)
            {
                throw new ParserException("Syntax error in query '" + query + "'");
            }

            var name = fields[0];
            var value = fields[1];
            var rule = name switch
            {
                "field" => new FieldIdRule((uint)ParseNumber("field", value)),
                "template" => new TemplateRule((uint)ParseNumber("template", value)),
                "groups" => new AccessGroupsRule(ParseNumber("groups", value)),
                "created" => new CreatedFlagRule(ParseBoolean("created", value)),
                "owner" => value == "room" ? (Rule)new RoomOwnerRule() : new UserOwnerRule((uint)ParseNumber("owner", value)),
                "id" => new IdRule((uint)ParseNumber("id", value)),
                _ => throw new ParserException("Field '" + name + "' not recognized")
            };
            if (not)
            {
                rule = new NotRule(rule);
            }

            return new RuleTerm(rule);
        }

        private static ulong ParseNumber(string field, string value)
        {
            try
            {
                return ulong.Parse(value);
            }
            catch (Exception)
            {
                throw new ParserException("Rule '" + field + "' must contains number but contains '" + value + "'");
            }
        }

        private static bool ParseBoolean(string field, string value)
        {
            try
            {
                return bool.Parse(value);
            }
            catch (Exception)
            {
                throw new ParserException("Rule '" + field + "' must contains true or false but contains '" + value + "'");
            }
        }
    }

    /// <summary>
    /// Промежуточный результат парсинга
    /// </summary>
    internal interface Term
    {
    }

    internal class RuleTerm : Term
    {
        internal readonly Rule rule;

        public RuleTerm(Rule rule)
        {
            this.rule = rule;
        }
    }

    internal class AndTerm : Term
    {
    }

    internal class OrTerm : Term
    {
    }

    public class ParserException : Exception
    {
        public ParserException(string message) : base(message)
        {
        }
    }
}