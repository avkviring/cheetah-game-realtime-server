using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using UnityEditor;
using UnityEngine.UIElements;

namespace Cheetah.Matches.Relay.Editor.UIElements.Table
{
    /// <summary>
    /// Табличное отображение текстовых данных
    /// </summary>
    public class TableElement : VisualElement
    {
        public class TableElementUxmlTraits : UxmlTraits
        {
        }

        public new class UxmlFactory : UxmlFactory<TableElement, TableElementUxmlTraits>
        {
        }

        private class Column
        {
            public readonly string title;
            public readonly int? minWidth;
            public readonly int? maxWidth;
            public readonly float? flexGrow;
            public readonly Func<object, string> toContent;

            public Column(string title, int? minWidth, int? maxWidth, float? flexGrow, Func<object, string> toContent)
            {
                this.title = title;
                this.minWidth = minWidth;
                this.maxWidth = maxWidth;
                this.flexGrow = flexGrow;
                this.toContent = toContent;
            }
        }

        private readonly List<Column> columns = new List<Column>();
        private readonly ListView table;
        private bool autoScroll;
        private IEnumerable<object> selectedItems = new List<object>();
        private IComparer selectItemComparer;
        private IComparer orderComparer;
        private event Action<IEnumerable<object>> OnSelectionChangeAction;

        public TableElement()
        {
            var uiAsset =
                AssetDatabase.LoadAssetAtPath<VisualTreeAsset>("Packages/games.cheetah.matches.relay/Editor/UIElements/Table/TableElement.uxml");
            uiAsset.CloneTree(this);
            table = this.Q<ListView>("table");
            table.makeItem = CreateRow;
            table.bindItem = BindItem;
            table.onSelectionChange += OnSelectItems;
            table.selectionType = SelectionType.None;
            table.showAlternatingRowBackgrounds = AlternatingRowBackground.ContentOnly;
        }

        private void OnSelectItems(IEnumerable<object> items)
        {
            selectedItems = items;
            OnSelectionChangeAction?.Invoke(items);
        }

        public void RegisterSelectedListener(Action<IEnumerable<object>> action)
        {
            OnSelectionChangeAction += action;
            table.selectionType = SelectionType.Single;
        }

        public void AddColumn(string title, int? minWidth, int? maxWidth, float? flexGrow, Func<object, string> toContent)
        {
            var column = new Column(title, minWidth, maxWidth, flexGrow, toContent);
            columns.Add(column);
            CreateHead(column);
        }


        public void EnableAutoScroll()
        {
            var scrollView = table.Q<ScrollView>();
            var scroller = scrollView.verticalScroller;
            scroller.valueChanged += value => { autoScroll = (scroller.highValue - scroller.value) < 5; };
        }

        private void CreateHead(Column column)
        {
            var header = this.Q<VisualElement>("headers");
            var head = new Label
            {
                text = column.title,
            };
            if (columns.Count > 1)
            {
                head.style.borderLeftWidth = 1;
            }

            SetupSize(head, column);
            head.AddToClassList("head");
            header.Add(head);
        }

        private void SetupSize(Label label, Column column)
        {
            if (column.minWidth != null)
            {
                label.style.minWidth = (StyleLength)column.minWidth;
            }

            if (column.maxWidth != null)
            {
                label.style.maxWidth = (StyleLength)column.maxWidth;
            }

            if (column.flexGrow != null)
            {
                label.style.flexGrow = (StyleFloat)column.flexGrow;
            }
        }

        private VisualElement CreateRow()
        {
            var box = new VisualElement();
            box.AddToClassList("table-row");
            var first = true;
            foreach (var column in columns)
            {
                var label = new Label();
                SetupSize(label, column);
                label.AddToClassList("table-cell");
                box.Add(label);
                if (!first)
                {
                    label.style.borderLeftWidth = 1;
                }

                first = false;
            }

            return box;
        }

        private void BindItem(VisualElement element, int row)
        {
            var item = table.itemsSource[row];
            var index = 0;
            foreach (var column in columns)
            {
                var label = element.ElementAt(index) as Label;
                label.text = column.toContent(item);
                index++;
            }
        }

        /// <summary>
        /// Установить компаратор для восстановления выделенного элемента при переустановке данных
        /// </summary>
        public void SetSelectItemComparer(IComparer comparer)
        {
            selectItemComparer = comparer;
        }

        /// <summary>
        /// Компаратор для сортировки элементов
        /// </summary>
        /// <param name="orderComparer"></param>
        public void SetOrderComparer(IComparer orderComparer)
        {
            this.orderComparer = orderComparer;
        }

        public void SetData(IList items)
        {
            if (orderComparer != null)
            {
                ArrayList.Adapter(items).Sort(orderComparer);
            }

            var selectedIndex = selectedItems.Any() ? FindItemIndex(items, selectedItems.First()) : -1;
            table.itemsSource = items;
            table.Rebuild();
            table.SetSelection(selectedIndex);
            if (selectedIndex > 0)
            {
                table.ScrollToItem(selectedIndex);
            }

            OnSelectionChangeAction?.Invoke(selectedItems);
        }

        private int FindItemIndex(IList items, object item)
        {
            if (selectItemComparer == null)
            {
                return items.IndexOf(item);
            }

            for (var i = 0; i < items.Count; i++)
            {
                if (selectItemComparer.Compare(items[i], item) == 0)
                {
                    return i;
                }
            }

            return -1;
        }

        public void Reset()
        {
            table.itemsSource = new List<object>();
            table.Clear();
            var header = this.Q<VisualElement>("headers");
            header.Clear();
            columns.Clear();
        }

        public void Update()
        {
            table.Rebuild();
            if (!autoScroll)
            {
                return;
            }

            table.ScrollToItem(-1);
        }
    }
}